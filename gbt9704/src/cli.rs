//! 命令行入口.
//!
//! 设计: 不引入 clap 等参数库, 参数面很小, 手写解析足够, 也便于离线构建.
//! 退出码: 0 成功; 1 校验失败; 2 用法/解析/IO 错误.

use std::fs;
use std::io::{self, Write};
use std::process::ExitCode;

use crate::gbt9704::{self, elements, fonts, geometry};
use crate::json::{self, Value};
use crate::mapping;
use crate::model::GbDocument;

const HELP: &str = r#"gbt9704 - GB/T 9704-2012 党政机关公文格式 -> Akoma Ntoso 3.1 生成器

用法:
  gbt9704 example [-o FILE]                 输出示例输入 JSON
  gbt9704 build -i FILE [-o FILE] [--ns 3.0|3.1]
                                            从 JSON 生成 AKN 3.1 XML (缺省写 stdout)
                                            --ns 3.0(默认): 官方 XSD 命名空间, 可用 akomantoso30.xsd 校验
                                            --ns 3.1: OASIS v2.0 命名空间文档口径, 需 adapted schema
  gbt9704 render -i FILE [-o FILE] [--dpi N] [--fonts DIR] [--single]
                                            阶段 2: 渲染为 A4 公文 PNG (缺省 200 dpi, 缺省 out-<页>.png)
                                            -o 单页时就是该文件, 多页时在扩展名前追加 -pN
                                            --single: 不分页, 输出一张连续长页(适合截图展示)
                                            --fonts DIR: 字体目录, 缺省 ./fonts 或 crate 根 fonts/
  gbt9704 validate -i FILE                  解析并校验输入 JSON
  gbt9704 elements [--format md|json] [-o FILE]
                                            列出标准格式要素注册表
  gbt9704 mapping [--format md|json] [-o FILE]
                                            列出标准要素 -> AKN 映射表
  gbt9704 geometry                          打印版面几何常量
  gbt9704 fonts                             打印字体要求与候选字体
  gbt9704 --help | -h                       显示本帮助
  gbt9704 --version                         版本号

退出码:
  0 成功; 1 校验失败; 2 用法/解析/IO 错误
"#;

/// 命令行主入口.
pub fn run(args: impl Iterator<Item = String>) -> ExitCode {
    let args: Vec<String> = args.collect();
    let code = match dispatch(&args) {
        Ok(code) => code,
        Err(message) => {
            eprintln!("错误: {message}");
            eprintln!("\n{}", HELP);
            2
        }
    };
    ExitCode::from(code)
}

fn dispatch(args: &[String]) -> Result<u8, String> {
    let Some(command) = args.first().map(String::as_str) else {
        println!("{HELP}");
        return Ok(0);
    };
    match command {
        "--help" | "-h" | "help" => {
            println!("{HELP}");
            Ok(0)
        }
        "--version" | "-V" => {
            println!("gbt9704 {}", env!("CARGO_PKG_VERSION"));
            Ok(0)
        }
        "example" => {
            let options = Options::parse(&args[1..])?;
            write_or_stdout(
                options.output.as_deref(),
                crate::model::EXAMPLE_JSON.as_bytes(),
            )?;
            Ok(0)
        }
        "build" => build(&Options::parse(&args[1..])?),
        "render" => render(&Options::parse(&args[1..])?),
        "validate" => validate(&Options::parse(&args[1..])?),
        "elements" => {
            let options = Options::parse(&args[1..])?;
            let format = options.format.as_deref().unwrap_or("md");
            let text = match format {
                "md" | "markdown" => render_elements_markdown(),
                "json" => render_elements_json(),
                other => return Err(format!("elements 不支持 --format {other:?}")),
            };
            write_or_stdout(options.output.as_deref(), text.as_bytes())?;
            Ok(0)
        }
        "mapping" => {
            let options = Options::parse(&args[1..])?;
            let format = options.format.as_deref().unwrap_or("md");
            let text = match format {
                "md" | "markdown" => mapping::render_markdown(),
                "json" => mapping::render_json(),
                other => return Err(format!("mapping 不支持 --format {other:?}")),
            };
            write_or_stdout(options.output.as_deref(), text.as_bytes())?;
            Ok(0)
        }
        "geometry" => {
            print_geometry();
            Ok(0)
        }
        "fonts" => {
            print_fonts();
            Ok(0)
        }
        other => Err(format!("未知命令 {other:?}")),
    }
}

fn build(options: &Options) -> Result<u8, String> {
    let input = options.input.as_deref().ok_or("build 需要 -i FILE")?;
    let text = fs::read_to_string(input).map_err(|error| format!("读取 {input} 失败: {error}"))?;
    let value = json::parse(&text).map_err(|error| format!("{input}: {error}"))?;
    let document = GbDocument::from_json(&value).map_err(|error| format!("{input}: {error}"))?;
    let diagnostics = document.validate();
    for diagnostic in &diagnostics {
        eprintln!("{diagnostic}");
    }
    let mode = match options.ns.as_deref() {
        Some(id) => crate::akn::NsMode::from_id(id)
            .ok_or_else(|| format!("--ns 只支持 3.0/3.1, 实际为 {id:?}"))?,
        None => crate::akn::NsMode::Official30,
    };
    match crate::akn::serialize_with(&document, mode) {
        Ok(xml) => {
            write_or_stdout(options.output.as_deref(), xml.as_bytes())?;
            let summary = format!(
                "{} namespace={} schema={}",
                crate::akn::summary(&document, &xml),
                mode.id(),
                mode.schema_file()
            );
            if let Some(output) = &options.output {
                eprintln!("已写入 {output}: {summary}");
            } else {
                eprintln!("{summary}");
            }
            Ok(0)
        }
        Err(errors) => Err(format!("存在 {} 个阻断性错误, 未生成 XML", errors.len())),
    }
}

/// 阶段 2: 渲染 PNG.
fn render(options: &Options) -> Result<u8, String> {
    let input = options.input.as_deref().ok_or("render 需要 -i FILE")?;
    let text = fs::read_to_string(input).map_err(|error| format!("读取 {input} 失败: {error}"))?;
    let value = json::parse(&text).map_err(|error| format!("{input}: {error}"))?;
    let document = GbDocument::from_json(&value).map_err(|error| format!("{input}: {error}"))?;
    for diagnostic in &document.validate() {
        if diagnostic.severity == crate::model::Severity::Error {
            return Err(format!("{diagnostic}"));
        }
        eprintln!("{diagnostic}");
    }
    let render_options = crate::render::RenderOptions {
        dpi: options.dpi.unwrap_or(200.0),
        fonts_dir: options.fonts.as_deref().map(std::path::PathBuf::from),
        continuous: options.continuous,
    };
    let pages = crate::render::render(&document, &render_options)?;
    let base = options
        .output
        .clone()
        .unwrap_or_else(|| default_render_output(input));
    for page in &pages {
        let path = page_output_path(&base, pages.len(), page.number);
        fs::write(&path, &page.png)
            .map_err(|error| format!("写入 {} 失败: {error}", path.display()))?;
        eprintln!(
            "已写入 {}: 第 {}/{} 页 {}×{} px @ {:.0} dpi",
            path.display(),
            page.number,
            pages.len(),
            page.width_px,
            page.height_px,
            render_options.dpi
        );
    }
    Ok(0)
}

/// 缺省输出名: 输入文件的词干 + .png.
fn default_render_output(input: &str) -> String {
    let stem = std::path::Path::new(input)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("document");
    format!("{stem}.png")
}

/// 多页时在扩展名前插入 `-pN`, 单页时就是 base 本身.
fn page_output_path(base: &str, page_count: usize, page: u32) -> std::path::PathBuf {
    let path = std::path::Path::new(base);
    if page_count <= 1 {
        return path.to_path_buf();
    }
    let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or("page");
    let extension = path.extension().and_then(|ext| ext.to_str()).unwrap_or("png");
    path.with_file_name(format!("{stem}-p{page}.{extension}"))
}

fn validate(options: &Options) -> Result<u8, String> {
    let input = options.input.as_deref().ok_or("validate 需要 -i FILE")?;
    let text = fs::read_to_string(input).map_err(|error| format!("读取 {input} 失败: {error}"))?;
    let value = json::parse(&text).map_err(|error| format!("{input}: {error}"))?;
    let document = GbDocument::from_json(&value).map_err(|error| format!("{input}: {error}"))?;
    let diagnostics = document.validate();
    for diagnostic in &diagnostics {
        println!("{diagnostic}");
    }
    let errors = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == crate::model::Severity::Error)
        .count();
    println!(
        "{}: {} 条错误, {} 条警告",
        input,
        errors,
        diagnostics.len() - errors
    );
    Ok(if errors == 0 { 0 } else { 1 })
}

/// 命令行选项.
#[derive(Default)]
struct Options {
    input: Option<String>,
    output: Option<String>,
    format: Option<String>,
    ns: Option<String>,
    /// 渲染分辨率(每英寸像素).
    dpi: Option<f64>,
    /// 渲染字体目录.
    fonts: Option<String>,
    /// 连续长页模式.
    continuous: bool,
}

impl Options {
    fn parse(args: &[String]) -> Result<Self, String> {
        let mut options = Self::default();
        let mut index = 0;
        while index < args.len() {
            match args[index].as_str() {
                "-i" | "--input" => {
                    index += 1;
                    options.input = Some(args.get(index).ok_or("选项 -i 缺少参数")?.clone());
                }
                "-o" | "--output" => {
                    index += 1;
                    options.output = Some(args.get(index).ok_or("选项 -o 缺少参数")?.clone());
                }
                "--format" => {
                    index += 1;
                    options.format = Some(args.get(index).ok_or("选项 --format 缺少参数")?.clone());
                }
                "--ns" => {
                    index += 1;
                    options.ns = Some(args.get(index).ok_or("选项 --ns 缺少参数")?.clone());
                }
                "--dpi" => {
                    index += 1;
                    let value = args.get(index).ok_or("选项 --dpi 缺少参数")?;
                    options.dpi = Some(
                        value
                            .parse()
                            .map_err(|_| format!("--dpi 需要数字, 实际为 {value:?}"))?,
                    );
                }
                "--fonts" => {
                    index += 1;
                    options.fonts = Some(args.get(index).ok_or("选项 --fonts 缺少参数")?.clone());
                }
                "--single" | "--continuous" => options.continuous = true,
                other => return Err(format!("未知选项 {other:?}")),
            }
            index += 1;
        }
        Ok(options)
    }
}

fn write_or_stdout(path: Option<&str>, bytes: &[u8]) -> Result<(), String> {
    match path {
        Some(path) => fs::write(path, bytes).map_err(|error| format!("写入 {path} 失败: {error}")),
        None => {
            let mut stdout = io::stdout().lock();
            stdout
                .write_all(bytes)
                .and_then(|()| stdout.flush())
                .map_err(|error| format!("写 stdout 失败: {error}"))
        }
    }
}

fn render_elements_markdown() -> String {
    let mut out = String::new();
    out.push_str("# GB/T 9704-2012 格式要素注册表\n\n");
    out.push_str("> 由 `cargo run -p gbt9704 -- elements --format md` 生成; 代码侧事实来源见 `src/gbt9704/elements.rs`。\n\n");
    for part in [
        elements::Part::Paper,
        elements::Part::Layout,
        elements::Part::Printing,
        elements::Part::Header,
        elements::Part::Body,
        elements::Part::Record,
        elements::Part::PageNumber,
        elements::Part::Table,
        elements::Part::ContentRules,
        elements::Part::Special,
        elements::Part::Sample,
    ] {
        let items: Vec<_> = elements::by_part(part).collect();
        if items.is_empty() {
            continue;
        }
        out.push_str(&format!("## {}\n\n", part.zh()));
        out.push_str("| id | 要素 | 条款 | 出现 | 字体 | 字号(pt) | 颜色 | 对齐 | 位置 | 规则 |\n");
        out.push_str("|---|---|---|---|---|---|---|---|---|---|\n");
        for item in items {
            out.push_str(&format!(
                "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
                item.id,
                item.name,
                item.clause,
                item.requirement.zh(),
                item.font.map(|slot| slot.standard_family()).unwrap_or("-"),
                item.font_size_pt
                    .map(|pt| format!("{pt}"))
                    .unwrap_or_else(|| "-".into()),
                item.color.hex(),
                item.align.zh(),
                escape_table(item.position),
                escape_table(item.rules),
            ));
        }
        out.push('\n');
    }
    out
}

fn escape_table(text: &str) -> String {
    text.replace('|', "\\|")
}

fn render_elements_json() -> String {
    let items: Vec<Value> = elements::ELEMENTS
        .iter()
        .map(|item| {
            Value::object(vec![
                ("id", Value::string(item.id)),
                ("name", Value::string(item.name)),
                ("part", Value::string(item.part.zh())),
                ("clause", Value::string(item.clause)),
                ("requirement", Value::string(item.requirement.id())),
                (
                    "font",
                    item.font
                        .map(|slot| Value::string(slot.id()))
                        .unwrap_or(Value::Null),
                ),
                (
                    "font_size_pt",
                    item.font_size_pt.map(Value::Number).unwrap_or(Value::Null),
                ),
                ("color", Value::string(item.color.hex())),
                ("align", Value::string(item.align.id())),
                ("position", Value::string(item.position)),
                ("spacing", Value::string(item.spacing)),
                ("rules", Value::string(item.rules)),
            ])
        })
        .collect();
    json::to_pretty(&Value::Array(items))
}

fn print_geometry() {
    println!("GB/T 9704-2012 版面几何");
    println!(
        "纸张: A4 {} mm × {} mm",
        geometry::PAPER_WIDTH_MM,
        geometry::PAPER_HEIGHT_MM
    );
    println!(
        "页边: 天头 {:.1} mm(±1) / 订口 {:.1} mm(±1) / 切口 {:.1} mm / 地脚 {:.1} mm",
        geometry::MARGIN_TOP_MM,
        geometry::MARGIN_LEFT_MM,
        geometry::MARGIN_RIGHT_MM,
        geometry::MARGIN_BOTTOM_MM
    );
    println!(
        "版心: {} mm × {} mm = {:.1} pt × {:.1} pt",
        geometry::TEXT_WIDTH_MM,
        geometry::TEXT_HEIGHT_MM,
        geometry::mm_to_pt(geometry::TEXT_WIDTH_MM),
        geometry::mm_to_pt(geometry::TEXT_HEIGHT_MM)
    );
    println!(
        "网格: {} 行 × {} 字; 字距 {:.4} mm ({:.4} pt); 行距 {:.4} mm ({:.4} pt)",
        geometry::LINES_PER_PAGE,
        geometry::CHARS_PER_LINE,
        geometry::char_pitch_mm(),
        geometry::char_pitch_pt(),
        geometry::line_pitch_mm(),
        geometry::line_pitch_pt()
    );
    println!(
        "术语定义: 1 字 = {} pt; 1 行 = {} pt = {:.4} mm (与网格行距不同, 见 geometry 注释)",
        geometry::SAN_HAO_PT,
        geometry::line_unit_pt(),
        geometry::line_unit_mm()
    );
    println!(
        "字号: 2 号 {} pt / 3 号 {} pt / 4 号 {} pt",
        22.0,
        geometry::SAN_HAO_PT,
        geometry::PAGE_NUMBER_FONT_PT
    );
    println!(
        "版记线: 粗 {:.2} mm / 细 {:.2} mm; 页码一字线上距版心下边缘 {:.1} mm; 页码字体 {:.0} pt",
        geometry::RECORD_LINE_THICK_MM,
        geometry::RECORD_LINE_THIN_MM,
        geometry::PAGE_NUMBER_LINE_OFFSET_MM,
        geometry::PAGE_NUMBER_FONT_PT
    );
    println!(
        "标准未规定/图测默认: 页码一字线 {:.2} mm; 发文机关标志 {:.0}–{:.0} pt(默认 {:.0}); 红分隔线厚约 {:.1} mm",
        geometry::PAGE_NUMBER_DASH_WIDTH_MM,
        geometry::ISSUER_MARK_MIN_PT,
        geometry::ISSUER_MARK_MAX_PT,
        geometry::ISSUER_MARK_DEFAULT_PT,
        geometry::HEADER_RED_LINE_THICKNESS_HINT_MM
    );
    println!(
        "特定格式: 发文机关标志上边缘至版心上边缘 {:.0} mm; 命令 {:.0} mm; 信函上边距 {:.0} mm, 双线长 {:.0} mm",
        geometry::ISSUER_MARK_TOP_FROM_TEXT_MM,
        geometry::ORDER_MARK_TOP_FROM_TEXT_MM,
        geometry::LETTER_MARK_TOP_FROM_PAPER_MM,
        geometry::LETTER_RED_LINE_WIDTH_MM
    );
    println!(
        "颜色: 黑 {} / 红 {} (红色油墨 CMYK {:.1},{:.1},{:.1},{:.1})",
        geometry::BLACK.hex(),
        geometry::RED.hex(),
        geometry::RED_CMYK.0,
        geometry::RED_CMYK.1,
        geometry::RED_CMYK.2,
        geometry::RED_CMYK.3
    );
    println!("标准: {} {}", gbt9704::STANDARD_ID, gbt9704::STANDARD_NAME);
}

fn print_fonts() {
    println!("GB/T 9704-2012 字体要求");
    println!(
        "{:<14} {:<22} {:<8} {:<6} 用途/条款",
        "slot", "标准措辞", "字号pt", "颜色"
    );
    for requirement in fonts::REQUIREMENTS {
        println!(
            "{:<14} {:<22} {:<8} {:<6} {} ({})",
            requirement.slot.id(),
            requirement.standard_text,
            requirement
                .size_pt
                .map(|pt| format!("{pt}"))
                .unwrap_or_else(|| "未规定".into()),
            requirement.color.hex(),
            requirement.usage,
            requirement.clause
        );
    }
    println!("\n候选字体:");
    println!(
        "{:<14} {:<42} {:<14} 可再分发/说明",
        "slot", "family", "license"
    );
    for candidate in fonts::CANDIDATES {
        println!(
            "{:<14} {:<42} {:<14} {} {}",
            candidate.slot.id(),
            candidate.family,
            candidate.license,
            if candidate.redistributable {
                "是"
            } else {
                "否"
            },
            candidate.note
        );
    }
    println!("\n字体文件与实测信息见 docs/fonts.md 与 fonts-manifest.json。");
}

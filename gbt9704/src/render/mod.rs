//! 阶段 2 渲染: 把 `GbDocument` 按 GB/T 9704-2012 排成 A4 公文页并输出 PNG.
//!
//! 覆盖范围(通用格式): 份号/密级/紧急程度、发文机关标志、发文字号、红色分隔线、标题、
//! 主送机关、正文(28 字 × 22 行网格)、附注、发文机关署名、成文日期、印章、页码、版记.
//! 本阶段不画附件与三种特定格式版面(命令/信函/纪要沿用通用格式的版头规则).
//!
//! 为什么用"排版计划 + 分页 + 逐页光栅化"三段式: 分页需要知道每行的真实高度(标题多行、
//! 印章占位), 所以先把元素排成带坐标的计划, 再决定哪些行落在哪一页, 最后统一交给
//! tiny-skia 画; 这比边排边画更容易保证"落款/印章不被切到版心之外".

pub mod draw;
pub mod font;
pub mod seal;
pub mod typeset;

use std::path::PathBuf;

use tiny_skia::{Pixmap, Transform};

use crate::gbt9704::fonts::FontSlot;
use crate::gbt9704::geometry;
use crate::model::{GbDocument, LayoutSpec, SealMode};

use font::FontBook;
use typeset::TextStyle;

/// 2 号字(标题/发文机关标志上限用).
const SIZE_2_PT: f64 = 22.0;
/// 3 号字(正文/发文字号/署名日期).
const SIZE_3_PT: f64 = 16.0;
/// 4 号字(页码/版记).
const SIZE_4_PT: f64 = 14.0;
/// 标题行距是字号的多少倍: 标准只要求"回行整齐", 2 号标题按 1.45 倍行距排两行才不挤.
const TITLE_LINE_FACTOR: f64 = 1.45;
/// 发文机关标志"文件"二字相对机关名称的放大倍数(标准不规定字号, 按式样图目测取值).
const MARK_SUFFIX_FACTOR: f64 = 1.28;
/// 圆形印章直径(mm): 机关印章的通行规格.
const SEAL_DIAMETER_MM: f64 = 42.0;

/// 红头/红线/印章的呈现色: 深红(约 #C00000).
///
/// 为什么不用 `geometry::RED`: 那是 6.2 红色油墨 CMYK(0,80,80,0) 的朴素 sRGB 换算
/// (#FF3333), 屏幕上偏橙; 印刷/式样图的实际观感接近 #C00000, 渲染层按观感取值.
pub(crate) fn ink_red() -> tiny_skia::Color {
    font::color(0xC0, 0x00, 0x00, 255)
}

/// 渲染选项.
#[derive(Clone, Debug)]
pub struct RenderOptions {
    /// 输出分辨率(每英寸像素).
    pub dpi: f64,
    /// 字体目录; 为空时自动寻找 `$GBT9704_FONTS` / `./fonts` / crate 根 `fonts/`.
    pub fonts_dir: Option<PathBuf>,
    /// 连续长页: 不分页, 页面高度随内容增长(适合截图/海报式展示, 不打印).
    pub continuous: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            dpi: 200.0,
            fonts_dir: None,
            continuous: false,
        }
    }
}

/// 一页渲染结果.
pub struct RenderedPage {
    /// 页码(从 1 开始).
    pub number: u32,
    pub width_px: u32,
    pub height_px: u32,
    /// PNG 字节.
    pub png: Vec<u8>,
}

/// 页面几何(mm). 默认值就是标准 5.2.1/5.2.3; `layout` 覆盖由输入 JSON 决定.
#[derive(Clone, Copy, Debug)]
struct Metrics {
    margin_top: f64,
    margin_left: f64,
    text_w: f64,
    text_h: f64,
    line_pitch: f64,
    char_pitch: f64,
    paper_h: f64,
}

impl Metrics {
    fn from_layout(layout: &LayoutSpec) -> Self {
        Self {
            margin_top: layout.margin_top_mm,
            margin_left: layout.margin_left_mm,
            text_w: layout.text_width_mm,
            text_h: layout.text_height_mm,
            line_pitch: layout.line_pitch_mm,
            char_pitch: layout.char_pitch_mm,
            paper_h: geometry::PAPER_HEIGHT_MM,
        }
    }

    fn text_left(&self) -> f64 {
        self.margin_left
    }

    fn text_right(&self) -> f64 {
        self.margin_left + self.text_w
    }

    fn text_top(&self) -> f64 {
        self.margin_top
    }

    fn text_bottom(&self) -> f64 {
        self.margin_top + self.text_h
    }

    /// 版心的水平中心.
    fn text_center_x(&self) -> f64 {
        self.margin_left + self.text_w / 2.0
    }

    /// 每页正文可用行数(第 1 页从 `body_start` 起算).
    fn line_capacity(&self, start_y: f64) -> usize {
        (((self.text_bottom() - start_y) / self.line_pitch) + 1e-6)
            .floor()
            .max(0.0) as usize
    }
}

/// 版头排版计划(所有 y 都是纸面坐标 mm).
struct HeaderPlan {
    /// 份号/密级/紧急程度, 依次占版心左上角第 1~3 行.
    extras: Vec<String>,
    /// 发文机关标志各行: (文字, 字号 pt).
    mark_lines: Vec<(String, f64)>,
    mark_block_top: f64,
    /// 发文字号文字; None 表示不排.
    doc_number: Option<String>,
    /// 签发人一栏文字(上行文).
    signers: Option<String>,
    doc_number_baseline: f64,
    red_line_top: f64,
    title_style: TextStyle,
    /// 标题折行后的各行文字(居中排).
    title_lines: Vec<String>,
    title_baselines: Vec<f64>,
    /// 主送机关折行后的各行文字(回行顶格).
    addr_lines: Vec<String>,
    addr_baseline: f64,
    /// 第 1 页正文第一行的网格顶部.
    body_start: f64,
}

/// 落款计划(相对最后一页正文末尾的坐标, 分页时才能定下来).
struct SignaturePlan {
    name: Option<String>,
    date: String,
    seal: Option<seal::SealSpec>,
    note: Option<String>,
}

/// 已排好版、可以逐页画的文档.
struct Prepared<'a> {
    doc: &'a GbDocument,
    metrics: Metrics,
    header: HeaderPlan,
    /// 正文行及其样式.
    body: Vec<(typeset::Line, TextStyle)>,
    signature: SignaturePlan,
}

/// 渲染整份公文, 返回每一页的 PNG.
pub fn render(document: &GbDocument, options: &RenderOptions) -> Result<Vec<RenderedPage>, String> {
    if !(30.0..=1200.0).contains(&options.dpi) {
        return Err(format!("--dpi 超出合理范围 30~1200: {}", options.dpi));
    }
    let book = FontBook::discover(options.fonts_dir.as_deref())?;
    let metrics = Metrics::from_layout(&document.layout);
    let prepared = prepare(document, &book, metrics)?;
    if options.continuous {
        let page = render_continuous(&book, &prepared, options)?;
        Ok(vec![page])
    } else {
        let plans = paginate(&prepared)?;
        let mut pages = Vec::with_capacity(plans.len());
        for plan in plans {
            pages.push(render_page(&book, &prepared, &plan, options)?);
        }
        Ok(pages)
    }
}

// ---------------------------------------------------------------------------
// 排版计划
// ---------------------------------------------------------------------------

fn prepare<'a>(
    document: &'a GbDocument,
    book: &FontBook,
    metrics: Metrics,
) -> Result<Prepared<'a>, String> {
    let header = plan_header(document, book, &metrics)?;

    let mut body = Vec::new();
    for block in &document.body {
        let style = TextStyle::new(block.font_slot(), SIZE_3_PT);
        let text = body_block_text(block);
        if text.trim().is_empty() {
            continue;
        }
        let lines = typeset::wrap(book, style, &text, metrics.text_w, 2.0 * metrics.char_pitch)?;
        for line in lines {
            body.push((line, style));
        }
    }

    let signature = plan_signature(document);
    Ok(Prepared {
        doc: document,
        metrics,
        header,
        body,
        signature,
    })
}

/// 正文块的排版文字: 层次标题把序数和文字拼成一行.
fn body_block_text(block: &crate::model::BodyBlock) -> String {
    match &block.num {
        Some(num) if !num.trim().is_empty() => {
            // "1." 这类阿拉伯数字序数后面留一个空格, 与式样图一致; "一、"/"(一)" 直接相连.
            if num.ends_with('.') {
                format!("{num} {}", block.text.clone())
            } else {
                format!("{num}{}", block.text)
            }
        }
        _ => block.text.clone(),
    }
}

fn plan_header(
    document: &GbDocument,
    book: &FontBook,
    metrics: &Metrics,
) -> Result<HeaderPlan, String> {
    // 7.2.1~7.2.3 份号/密级/紧急程度: 3 号黑体, 顶格, 自上而下各占一行.
    let mut extras = Vec::new();
    if let Some(copy_number) = &document.copy_number {
        extras.push(copy_number.clone());
    }
    if let Some(secrecy) = &document.secrecy {
        extras.push(secrecy.display());
    }
    if let Some(urgency) = &document.urgency {
        extras.push(urgency.clone());
    }

    // 7.2.4 发文机关标志: 红色小标宋, 上边缘距版心上边缘 35 mm.
    let mark_text = document.issuer.mark_text_for(document.format);
    let mark_lines = plan_issuer_mark(book, &mark_text, metrics.text_w)?;
    let mark_block_top = metrics.text_top() + geometry::ISSUER_MARK_TOP_FROM_TEXT_MM;
    let mut mark_bottom = mark_block_top;
    for (index, (_, pt)) in mark_lines.iter().enumerate() {
        let size = font::size_mm(*pt);
        let gap = if index == 0 { 0.0 } else { size * 0.30 };
        mark_bottom += gap + size;
    }

    // 7.2.5 发文字号: 发文机关标志下空二行, 居中(上行文居左空一字, 与签发人同行).
    let three_mm = font::size_mm(SIZE_3_PT);
    let doc_number_top = mark_bottom + 2.0 * metrics.line_pitch;
    let doc_number_baseline = doc_number_top + font::BASELINE_EM * three_mm;
    let doc_number = document.doc_number.as_ref().map(|number| number.display());
    let signers = if document.signers.is_empty() {
        None
    } else {
        let names: Vec<&str> = document
            .signers
            .iter()
            .map(|signer| signer.name.as_str())
            .collect();
        Some(format!("签发人：{}", names.join("　")))
    };

    // 7.2.7 红色分隔线: 发文字号之下 4 mm, 与版心等宽.
    let red_line_top = doc_number_top + three_mm + geometry::HEADER_RED_LINE_GAP_MM;

    // 7.3.1 标题: 红色分隔线下空二行, 2 号小标宋, 居中.
    let title_style = TextStyle::new(FontSlot::Title, SIZE_2_PT);
    let title_top =
        red_line_top + geometry::HEADER_RED_LINE_THICKNESS_HINT_MM + 2.0 * metrics.line_pitch;
    let title_lines: Vec<String> =
        balanced_wrap(book, title_style, &document.title, metrics.text_w)?
            .into_iter()
            .map(|line| line.text)
            .collect();
    let title_pitch = title_style.size_mm * TITLE_LINE_FACTOR;
    let mut title_baselines = Vec::with_capacity(title_lines.len());
    let mut title_bottom = title_top;
    for index in 0..title_lines.len() {
        title_baselines
            .push(title_top + index as f64 * title_pitch + font::BASELINE_EM * title_style.size_mm);
        title_bottom = title_top + index as f64 * title_pitch + title_style.size_mm;
    }

    // 7.3.2 主送机关: 标题下空一行, 3 号仿宋, 居左顶格, 回行仍顶格.
    let addr_lines: Vec<String> = if document.addressees.is_empty() {
        Vec::new()
    } else {
        let text = format!("{}：", document.addressees.join("，"));
        typeset::wrap(
            book,
            TextStyle::new(FontSlot::Body, SIZE_3_PT),
            &text,
            metrics.text_w,
            0.0,
        )?
        .into_iter()
        .map(|line| line.text)
        .collect()
    };
    let addr_top = title_bottom + metrics.line_pitch;
    let addr_baseline = addr_top + font::BASELINE_EM * three_mm;

    // 7.3.3 正文: 主送机关下一行; 没有主送机关时从标题下空一行开始.
    let body_start = if addr_lines.is_empty() {
        title_bottom + metrics.line_pitch
    } else {
        addr_top
            + (addr_lines.len() as f64 - 1.0) * metrics.line_pitch
            + three_mm
            + metrics.line_pitch
    };

    Ok(HeaderPlan {
        extras,
        mark_lines,
        mark_block_top,
        doc_number,
        signers,
        doc_number_baseline,
        red_line_top,
        title_style,
        title_lines,
        title_baselines,
        addr_lines,
        addr_baseline,
        body_start,
    })
}

/// 标题折行: 式样图要求"梯形或菱形、词意完整", 所以不能简单按宽度贪心折行.
///
/// 做法: 先贪心折行得到行数; 若是两行, 在所有"两行都放得下"的断点里挑一个分数最低的:
/// 长度接近(梯形) + 断在分句处(及/和/与/经/因…之前)加分, 断在虚词之前或括号之后扣分,
/// 不拆外文单词. 三行以上退回"最小行宽二分"的均衡折行.
fn balanced_wrap(
    book: &FontBook,
    style: TextStyle,
    text: &str,
    max_width_mm: f64,
) -> Result<Vec<typeset::Line>, String> {
    let greedy = typeset::wrap(book, style, text, max_width_mm, 0.0)?;
    let count = greedy.len();
    if count <= 1 {
        return Ok(greedy);
    }
    if count == 2 {
        if let Some(lines) = best_two_line_split(book, style, text, max_width_mm)? {
            return Ok(lines);
        }
    }
    // 行数随行宽单调不增: 二分出"仍能保持同样行数"的最小行宽.
    let (mut low, mut high) = (0.0f64, max_width_mm);
    for _ in 0..40 {
        let mid = (low + high) / 2.0;
        if typeset::wrap(book, style, text, mid, 0.0)?.len() <= count {
            high = mid;
        } else {
            low = mid;
        }
    }
    typeset::wrap(book, style, text, high + style.size_mm * 0.02, 0.0)
}

/// 两行标题的最优断点: 返回 None 表示找不到比贪心更好的断法.
fn best_two_line_split(
    book: &FontBook,
    style: TextStyle,
    text: &str,
    max_width_mm: f64,
) -> Result<Option<Vec<typeset::Line>>, String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 3 {
        return Ok(None);
    }
    // 断在分句词之前的加分位置: 公文标题常在这些词处断行才通顺.
    const CLAUSE_START: &[char] = &[
        '及', '和', '与', '并', '而', '或', '以', '为', '在', '对', '将', '把', '从', '向', '于',
        '因', '由', '经', '就', '其', '该', '各', '凡', '此', '本', '所', '另', '同', '即', '且',
        '则', '均', '皆', '共',
    ];
    // 断在这些字之前最难看: 虚词单独起行、或把词切开后接虚词.
    const BAD_START: &[char] = &['的', '了', '着', '地', '得', '们', '是', '有'];
    let mut best: Option<(f64, Vec<typeset::Line>)> = None;
    for index in 1..chars.len() {
        let (previous, next) = (chars[index - 1], chars[index]);
        if previous.is_ascii_alphanumeric() && next.is_ascii_alphanumeric() {
            continue;
        }
        if matches!(
            next,
            '，' | '。' | '、' | '；' | '：' | '）' | '”' | '》' | '』'
        ) {
            continue;
        }
        if matches!(previous, '（' | '“' | '《' | '【' | '『') {
            continue;
        }
        let head: String = chars[..index].iter().collect();
        let tail: String = chars[index..].iter().collect();
        let head_width = typeset::measure(book, style, &head)?;
        let tail_width = typeset::measure(book, style, &tail)?;
        if head_width > max_width_mm + 0.5 || tail_width > max_width_mm + 0.5 {
            continue;
        }
        let mut score = (head_width - tail_width).abs() * 0.25;
        if BAD_START.contains(&next) {
            score += style.size_mm * 4.0;
        }
        if CLAUSE_START.contains(&next) {
            score -= style.size_mm * 2.0;
        }
        if best
            .as_ref()
            .is_none_or(|(best_score, _)| score < *best_score)
        {
            let lines = vec![
                typeset::Line {
                    text: head,
                    width_mm: head_width,
                    indent_mm: 0.0,
                    last: false,
                },
                typeset::Line {
                    text: tail,
                    width_mm: tail_width,
                    indent_mm: 0.0,
                    last: true,
                },
            ];
            best = Some((score, lines));
        }
    }
    Ok(best.map(|(_, lines)| lines))
}

/// 发文机关标志的断行与字号.
///
/// 标准只规定"以醒目、美观、庄重为原则"(7.2.4), 字号由机关按名称字数自定. 规则:
/// 1. 输入里有换行符就按换行分行, 各行同字号(取能放下最宽一行的字号);
/// 2. 名称+"文件"能一行放下且字号不小于建议下限时排一行;
/// 3. 否则把"文件/命令/令/纪要"后缀拆到第二行, 名称取能撑满版心的字号,
///    后缀按 `MARK_SUFFIX_FACTOR` 放大(式样图里"文件"二字明显大于机关名称).
fn plan_issuer_mark(
    book: &FontBook,
    text: &str,
    max_width_mm: f64,
) -> Result<Vec<(String, f64)>, String> {
    let max_pt = geometry::ISSUER_MARK_MAX_PT;
    let min_pt = geometry::ISSUER_MARK_MIN_PT;

    if text.contains('\n') {
        let lines: Vec<String> = text
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect();
        if lines.is_empty() {
            return Err("issuer.mark 没有任何可见文字".into());
        }
        let mut size = f64::INFINITY;
        for line in &lines {
            size = size.min(fit_pt(book, FontSlot::IssuerMark, line, max_width_mm)?);
        }
        let size = size.clamp(min_pt, max_pt);
        return Ok(lines.into_iter().map(|line| (line, size)).collect());
    }

    let full = fit_pt(book, FontSlot::IssuerMark, text, max_width_mm)?;
    if full >= min_pt {
        return Ok(vec![(text.to_string(), full.min(max_pt))]);
    }

    let (name, suffix) = split_mark_suffix(text);
    if !suffix.is_empty() && !name.is_empty() {
        let name_pt =
            fit_pt(book, FontSlot::IssuerMark, &name, max_width_mm)?.clamp(min_pt, max_pt);
        let suffix_pt = (name_pt * MARK_SUFFIX_FACTOR)
            .min(fit_pt(book, FontSlot::IssuerMark, &suffix, max_width_mm)?)
            .clamp(min_pt, max_pt);
        return Ok(vec![(name, name_pt), (suffix, suffix_pt)]);
    }
    Ok(vec![(text.to_string(), full.clamp(min_pt, max_pt))])
}

/// 拆出发文机关标志的后缀("文件"/"命令"/"令"/"纪要"), 返回(名称, 后缀).
fn split_mark_suffix(text: &str) -> (String, String) {
    for suffix in ["文件", "命令", "纪要", "令"] {
        if let Some(name) = text.strip_suffix(suffix)
            && !name.is_empty()
        {
            return (name.to_string(), suffix.to_string());
        }
    }
    (text.to_string(), String::new())
}

/// 求使 `text` 宽度正好等于 `max_width_mm` 的字号(pt).
fn fit_pt(book: &FontBook, slot: FontSlot, text: &str, max_width_mm: f64) -> Result<f64, String> {
    let face = book.face(slot)?;
    let probe_pt = 10.0;
    let width = font::grid_width_mm(&face, font::size_mm(probe_pt), text);
    if width <= 1e-9 {
        return Ok(probe_pt);
    }
    Ok(probe_pt * max_width_mm / width)
}

fn plan_signature(document: &GbDocument) -> SignaturePlan {
    let name = document
        .signature
        .issuer_names
        .first()
        .cloned()
        .or_else(|| document.issuer.names.first().cloned());
    let date = document
        .signature
        .date_text
        .clone()
        .or_else(|| crate::model::iso_date_to_zh(&document.signature.date))
        .unwrap_or_else(|| document.signature.date.clone());
    let seal = if document.signature.mode == SealMode::WithSeal {
        let first = document.signature.seals.first();
        Some(seal::SealSpec {
            text: first
                .and_then(|seal| seal.issuer.clone())
                .or_else(|| name.clone())
                .unwrap_or_default(),
            number: first.and_then(|seal| seal.number.clone()),
        })
    } else {
        None
    };
    SignaturePlan {
        name,
        date,
        seal,
        note: document.note.clone(),
    }
}

// ---------------------------------------------------------------------------
// 分页
// ---------------------------------------------------------------------------

/// 一页的内容计划.
struct PagePlan {
    number: u32,
    /// 本页正文行区间.
    body: std::ops::Range<usize>,
    /// 是否画版头(第 1 页).
    header: bool,
    /// 是否画落款/印章.
    signature: bool,
    /// 是否最后一页(版记画在这里).
    last: bool,
    /// 落款第一行(附注或署名)的网格顶部; `signature=true` 时有效.
    signature_top: f64,
}

/// 落款块底部(mm): 附注(可选) + 署名 + 成文日期 + 以日期为准居中的印章.
///
/// 与 `draw_signature` 使用同一套坐标, 分页判断才有意义.
fn signature_bottom_mm(plan: &SignaturePlan, metrics: &Metrics, signature_top: f64) -> f64 {
    let three_mm = font::size_mm(SIZE_3_PT);
    let cell_top = signature_top;
    let name_baseline = typeset::baseline_in_cell(cell_top, metrics.line_pitch, three_mm);
    let date_baseline =
        typeset::baseline_in_cell(cell_top + metrics.line_pitch, metrics.line_pitch, three_mm);
    let name_ink_top = name_baseline - font::BASELINE_EM * three_mm;
    let date_ink_bottom = date_baseline + font::DESCENT_EM * three_mm;
    let center_y = (name_ink_top + date_ink_bottom) / 2.0;
    let seal_bottom = center_y + SEAL_DIAMETER_MM / 2.0;
    let note_bottom = if plan.note.is_some() {
        cell_top + 2.0 * metrics.line_pitch + three_mm
    } else {
        seal_bottom
    };
    seal_bottom.max(note_bottom)
}

fn paginate(prepared: &Prepared<'_>) -> Result<Vec<PagePlan>, String> {
    let metrics = &prepared.metrics;
    let header = &prepared.header;
    let total = prepared.body.len();
    let mut plans: Vec<PagePlan> = Vec::new();
    let mut cursor = 0usize;
    let mut first = true;

    while cursor < total {
        let start_y = if first {
            header.body_start
        } else {
            metrics.text_top()
        };
        let capacity = metrics.line_capacity(start_y).max(1);
        let end = (cursor + capacity).min(total);
        plans.push(PagePlan {
            number: plans.len() as u32 + 1,
            body: cursor..end,
            header: first,
            signature: false,
            last: false,
            signature_top: 0.0,
        });
        cursor = end;
        first = false;
    }
    if plans.is_empty() {
        // 空正文(只有落款): 仍然要有一页放版头和落款.
        plans.push(PagePlan {
            number: 1,
            body: 0..0,
            header: true,
            signature: false,
            last: false,
            signature_top: 0.0,
        });
    }

    // 落款与印章必须落在版心内: 放不下就把最后一页的末尾几行挪到新页, 与落款同页
    // (7.3.5 要求印章下压署名与日期, 且"末页必须显示正文").
    loop {
        let last = plans.len() - 1;
        let body_end = line_bottom(prepared, metrics, &plans[last]);
        let top = body_end + metrics.line_pitch;
        let bottom = signature_bottom_mm(&prepared.signature, metrics, top);
        if bottom <= metrics.text_bottom() + 1e-6 {
            plans[last].signature = true;
            plans[last].signature_top = top;
            break;
        }
        // 挪最后一行到新页; 没有可挪的行就新开一页只放落款.
        let movable = !plans[last].body.is_empty();
        if movable {
            let mut moved = plans[last].body.clone();
            moved.end -= 1;
            let moved_line = moved.end;
            let page_range = plans[last].body.clone();
            plans[last].body = page_range.start..moved_line;
            plans.push(PagePlan {
                number: plans.len() as u32 + 1,
                body: moved_line..page_range.end,
                header: false,
                signature: false,
                last: false,
                signature_top: 0.0,
            });
        } else {
            plans.push(PagePlan {
                number: plans.len() as u32 + 1,
                body: 0..0,
                header: false,
                signature: false,
                last: false,
                signature_top: 0.0,
            });
        }
    }

    // 最后再按页序重编号并标记末页, 保证 1..n.
    let count = plans.len();
    for (index, plan) in plans.iter_mut().enumerate() {
        plan.number = index as u32 + 1;
        plan.last = index + 1 == count;
    }
    Ok(plans)
}

/// 某页正文最后一行的网格底部(mm); 无正文行时回退到正文起始位置.
fn line_bottom(prepared: &Prepared<'_>, metrics: &Metrics, page: &PagePlan) -> f64 {
    let start_y = if page.header {
        prepared.header.body_start
    } else {
        metrics.text_top()
    };
    if page.body.is_empty() {
        start_y
    } else {
        start_y + (page.body.end - page.body.start) as f64 * metrics.line_pitch
    }
}

// ---------------------------------------------------------------------------
// 逐页绘制
// ---------------------------------------------------------------------------

fn render_page(
    book: &FontBook,
    prepared: &Prepared<'_>,
    plan: &PagePlan,
    options: &RenderOptions,
) -> Result<RenderedPage, String> {
    let mut pixmap = new_pixmap(
        geometry::PAPER_WIDTH_MM,
        options.dpi,
        geometry::PAPER_HEIGHT_MM,
    )?;
    let transform = page_transform(options.dpi);
    let metrics = &prepared.metrics;
    let black = font::color(0, 0, 0, 255);
    let red = ink_red();

    if plan.header {
        draw_header(book, prepared, &mut pixmap, transform, black, red)?;
    }

    // 正文.
    let start_y = if plan.header {
        prepared.header.body_start
    } else {
        metrics.text_top()
    };
    for (offset, (line, style)) in prepared.body[plan.body.clone()].iter().enumerate() {
        let cell_top = start_y + offset as f64 * metrics.line_pitch;
        let baseline = typeset::baseline_in_cell(cell_top, metrics.line_pitch, style.size_mm);
        typeset::draw_line(
            &mut pixmap,
            book,
            *style,
            metrics.text_left(),
            baseline,
            line,
            black,
            Some(metrics.text_w),
            transform,
        )?;
    }

    if plan.signature {
        draw_signature(book, prepared, &mut pixmap, transform, plan.signature_top)?;
    }

    // 7.4 版记: 只画在最后一页, 贴版心下边缘.
    if plan.last && plans_has_record(prepared) {
        draw_record(
            book,
            prepared,
            &mut pixmap,
            transform,
            black,
            metrics.text_bottom(),
        )?;
    }
    // 7.5 页码: 版心下缘之下 7 mm.
    draw_page_number(book, prepared, plan, &mut pixmap, transform, black)?;

    let png = pixmap
        .encode_png()
        .map_err(|error| format!("PNG 编码失败: {error}"))?;
    Ok(RenderedPage {
        number: plan.number,
        width_px: pixmap.width(),
        height_px: pixmap.height(),
        png,
    })
}

/// 连续长页: 不分页, 页高 = 版头 + 全部正文 + 落款 + 下边距.
fn render_continuous(
    book: &FontBook,
    prepared: &Prepared<'_>,
    options: &RenderOptions,
) -> Result<RenderedPage, String> {
    let metrics = &prepared.metrics;
    let body_end = prepared.header.body_start + prepared.body.len() as f64 * metrics.line_pitch;
    let signature_top = body_end + metrics.line_pitch;
    let margin_bottom = metrics.paper_h - metrics.text_bottom();
    let content_bottom = signature_bottom_mm(&prepared.signature, metrics, signature_top);
    let height_mm = (content_bottom + margin_bottom).max(metrics.paper_h);

    let mut pixmap = new_pixmap(geometry::PAPER_WIDTH_MM, options.dpi, height_mm)?;
    let transform = page_transform(options.dpi);
    let black = font::color(0, 0, 0, 255);
    let red = ink_red();

    draw_header(book, prepared, &mut pixmap, transform, black, red)?;
    for (offset, (line, style)) in prepared.body.iter().enumerate() {
        let cell_top = prepared.header.body_start + offset as f64 * metrics.line_pitch;
        let baseline = typeset::baseline_in_cell(cell_top, metrics.line_pitch, style.size_mm);
        typeset::draw_line(
            &mut pixmap,
            book,
            *style,
            metrics.text_left(),
            baseline,
            line,
            black,
            Some(metrics.text_w),
            transform,
        )?;
    }
    draw_signature(book, prepared, &mut pixmap, transform, signature_top)?;
    if plans_has_record(prepared) {
        draw_record(
            book,
            prepared,
            &mut pixmap,
            transform,
            black,
            content_bottom + margin_bottom / 2.0,
        )?;
    }
    let png = pixmap
        .encode_png()
        .map_err(|error| format!("PNG 编码失败: {error}"))?;
    Ok(RenderedPage {
        number: 1,
        width_px: pixmap.width(),
        height_px: pixmap.height(),
        png,
    })
}

fn new_pixmap(width_mm: f64, dpi: f64, height_mm: f64) -> Result<Pixmap, String> {
    let scale = dpi / 25.4;
    let width = (width_mm * scale).round().max(1.0) as u32;
    let height = (height_mm * scale).round().max(1.0) as u32;
    let mut pixmap =
        Pixmap::new(width, height).ok_or_else(|| format!("页面像素尺寸非法: {width}×{height}"))?;
    pixmap.fill(tiny_skia::Color::WHITE);
    Ok(pixmap)
}

fn page_transform(dpi: f64) -> Transform {
    let scale = (dpi / 25.4) as f32;
    Transform::from_scale(scale, scale)
}

/// 画版头: 份号/密级/紧急程度、发文机关标志、发文字号、签发人、红色分隔线、标题、主送机关.
fn draw_header(
    book: &FontBook,
    prepared: &Prepared<'_>,
    pixmap: &mut Pixmap,
    transform: Transform,
    black: tiny_skia::Color,
    red: tiny_skia::Color,
) -> Result<(), String> {
    let metrics = &prepared.metrics;
    let header = &prepared.header;
    let three_mm = font::size_mm(SIZE_3_PT);
    let heiti = TextStyle::new(FontSlot::Heiti, SIZE_3_PT);

    // 份号/密级/紧急程度.
    for (index, text) in header.extras.iter().enumerate() {
        let cell_top = metrics.text_top() + index as f64 * metrics.line_pitch;
        let baseline = typeset::baseline_in_cell(cell_top, metrics.line_pitch, three_mm);
        typeset::draw_left(
            pixmap,
            book,
            heiti,
            metrics.text_left(),
            baseline,
            text,
            black,
            transform,
        )?;
    }

    // 发文机关标志: 逐行红色小标宋.
    let mut ink_top = header.mark_block_top;
    for (index, (text, pt)) in header.mark_lines.iter().enumerate() {
        let size = font::size_mm(*pt);
        if index > 0 {
            ink_top += size * 0.30;
        }
        let baseline = ink_top + font::BASELINE_EM * size;
        let style = TextStyle::new(FontSlot::IssuerMark, *pt);
        typeset::draw_centered(
            pixmap,
            book,
            style,
            metrics.text_center_x(),
            baseline,
            text,
            red,
            transform,
        )?;
        ink_top += size;
    }

    // 发文字号: 居中; 上行文居左空一字, 签发人居右空一字.
    let doc_style = TextStyle::new(FontSlot::Body, SIZE_3_PT);
    let baseline = header.doc_number_baseline;
    if let Some(number) = &header.doc_number {
        if header.signers.is_some() {
            typeset::draw_left(
                pixmap,
                book,
                doc_style,
                metrics.text_left() + metrics.char_pitch,
                baseline,
                number,
                black,
                transform,
            )?;
        } else {
            typeset::draw_centered(
                pixmap,
                book,
                doc_style,
                metrics.text_center_x(),
                baseline,
                number,
                black,
                transform,
            )?;
        }
    }
    if let Some(signers) = &header.signers {
        let kaiti = TextStyle::new(FontSlot::Kaiti, SIZE_3_PT);
        let width = typeset::measure(book, kaiti, signers)?;
        typeset::draw_left(
            pixmap,
            book,
            kaiti,
            metrics.text_right() - metrics.char_pitch - width,
            baseline,
            signers,
            black,
            transform,
        )?;
    }

    // 红色分隔线.
    draw::bar(
        pixmap,
        metrics.text_left(),
        header.red_line_top + geometry::HEADER_RED_LINE_THICKNESS_HINT_MM / 2.0,
        metrics.text_right(),
        geometry::HEADER_RED_LINE_THICKNESS_HINT_MM,
        red,
        transform,
    );

    // 标题: 多行居中, 行距按 1.45 倍字号.
    for (index, text) in header.title_lines.iter().enumerate() {
        let Some(baseline) = header.title_baselines.get(index) else {
            continue;
        };
        typeset::draw_centered(
            pixmap,
            book,
            header.title_style,
            metrics.text_center_x(),
            *baseline,
            text,
            black,
            transform,
        )?;
    }

    // 主送机关: 每行都顶格.
    for (index, text) in header.addr_lines.iter().enumerate() {
        let baseline = header.addr_baseline + index as f64 * metrics.line_pitch;
        typeset::draw_left(
            pixmap,
            book,
            doc_style,
            metrics.text_left(),
            baseline,
            text,
            black,
            transform,
        )?;
    }
    Ok(())
}

/// 画落款: 附注 → 发文机关署名 → 成文日期 → 印章.
fn draw_signature(
    book: &FontBook,
    prepared: &Prepared<'_>,
    pixmap: &mut Pixmap,
    transform: Transform,
    signature_top: f64,
) -> Result<(), String> {
    let metrics = &prepared.metrics;
    let plan = &prepared.signature;
    let three_mm = font::size_mm(SIZE_3_PT);
    let style = TextStyle::new(FontSlot::Body, SIZE_3_PT);
    let black = font::color(0, 0, 0, 255);
    let cell_top = signature_top;

    // 7.3.5.1 发文机关署名在成文日期之上, 以成文日期为准居中; 成文日期右空四字.
    let name_cell_top = cell_top;
    let name_baseline = typeset::baseline_in_cell(name_cell_top, metrics.line_pitch, three_mm);
    let date_baseline =
        typeset::baseline_in_cell(cell_top + metrics.line_pitch, metrics.line_pitch, three_mm);
    let date_width = typeset::measure(book, style, &plan.date)?;
    let date_right = metrics.text_right() - 4.0 * metrics.char_pitch;
    let date_x = date_right - date_width;
    typeset::draw_left(
        pixmap,
        book,
        style,
        date_x,
        date_baseline,
        &plan.date,
        black,
        transform,
    )?;

    let date_center = date_x + date_width / 2.0;
    // 署名排在日期之上, 行距是一行.
    if let Some(name) = &plan.name
        && !name.is_empty()
    {
        let width = typeset::measure(book, style, name)?;
        typeset::draw_left(
            pixmap,
            book,
            style,
            date_center - width / 2.0,
            name_baseline,
            name,
            black,
            transform,
        )?;
    }

    // 7.3.6 附注: 成文日期下一行, 左空二字.
    if let Some(note) = &plan.note {
        let baseline = typeset::baseline_in_cell(
            cell_top + 2.0 * metrics.line_pitch,
            metrics.line_pitch,
            three_mm,
        );
        typeset::draw_left(
            pixmap,
            book,
            style,
            metrics.text_left() + 2.0 * metrics.char_pitch,
            baseline,
            note,
            black,
            transform,
        )?;
    }

    if let Some(seal) = &plan.seal {
        // 印章端正、居中下压署名和日期: 圆心取署名墨迹顶到日期墨迹底的中间.
        let name_ink_top = name_baseline - font::BASELINE_EM * three_mm;
        let date_ink_bottom = date_baseline + font::DESCENT_EM * three_mm;
        let center_y = (name_ink_top + date_ink_bottom) / 2.0;
        seal::draw(
            pixmap,
            book,
            date_center,
            center_y,
            SEAL_DIAMETER_MM,
            seal,
            transform,
        )?;
    }
    Ok(())
}

/// 版记是否有内容.
fn plans_has_record(prepared: &Prepared<'_>) -> bool {
    !prepared.doc.cc.is_empty() || prepared.doc.printing.is_some()
}

/// 7.4 版记: 抄送机关与印发机关/日期, 首末线粗、行间线细, 贴版心下边缘.
fn draw_record(
    book: &FontBook,
    prepared: &Prepared<'_>,
    pixmap: &mut Pixmap,
    transform: Transform,
    black: tiny_skia::Color,
    anchor_bottom_mm: f64,
) -> Result<(), String> {
    let metrics = &prepared.metrics;
    let doc = prepared.doc;
    let style = TextStyle::new(FontSlot::Record, SIZE_4_PT);
    let size = style.size_mm;
    let mut rows: Vec<(String, String)> = Vec::new();
    if !doc.cc.is_empty() {
        rows.push((format!("抄送：{}。", doc.cc.join("，")), String::new()));
    }
    if let Some(printing) = &doc.printing {
        let date =
            crate::model::iso_date_to_zh(&printing.date).unwrap_or_else(|| printing.date.clone());
        rows.push((format!("{}", printing.issuer), date));
    }
    if rows.is_empty() {
        return Ok(());
    }
    let thick = geometry::RECORD_LINE_THICK_MM;
    let thin = geometry::RECORD_LINE_THIN_MM;
    let row_h = size * 1.55;
    let mut y = anchor_bottom_mm - thick / 2.0;
    draw::bar(
        pixmap,
        metrics.text_left(),
        y,
        metrics.text_right(),
        thick,
        black,
        transform,
    );
    for (index, (left, right)) in rows.iter().enumerate().rev() {
        let row_bottom = y - thick / 2.0;
        let baseline = row_bottom - row_h / 2.0 + font::BASELINE_EM * size - size / 2.0;
        let text = if right.is_empty() {
            left.clone()
        } else {
            left.clone()
        };
        typeset::draw_left(
            pixmap,
            book,
            style,
            metrics.text_left() + metrics.char_pitch,
            baseline,
            &text,
            black,
            transform,
        )?;
        if !right.is_empty() {
            let width = typeset::measure(book, style, right)?;
            typeset::draw_left(
                pixmap,
                book,
                style,
                metrics.text_right() - metrics.char_pitch - width,
                baseline,
                right,
                black,
                transform,
            )?;
        }
        if index > 0 {
            y = row_bottom - row_h;
            draw::bar(
                pixmap,
                metrics.text_left(),
                y,
                metrics.text_right(),
                if index == rows.len() - 1 { thick } else { thin },
                black,
                transform,
            );
        } else {
            y = row_bottom - row_h;
            draw::bar(
                pixmap,
                metrics.text_left(),
                y,
                metrics.text_right(),
                thick,
                black,
                transform,
            );
        }
    }
    Ok(())
}

/// 7.5 页码: 4 号宋体, "— N —", 单页居右空一字, 双页居左空一字, 一字线在版心下 7 mm.
fn draw_page_number(
    book: &FontBook,
    prepared: &Prepared<'_>,
    plan: &PagePlan,
    pixmap: &mut Pixmap,
    transform: Transform,
    black: tiny_skia::Color,
) -> Result<(), String> {
    if !prepared.doc.page.enabled {
        return Ok(());
    }
    let metrics = &prepared.metrics;
    let style = TextStyle::new(FontSlot::PageNumber, SIZE_4_PT);
    let text = format!("— {} —", plan.number);
    let width = typeset::measure(book, style, &text)?;
    let baseline = metrics.text_bottom()
        + geometry::PAGE_NUMBER_LINE_OFFSET_MM
        + font::BASELINE_EM * style.size_mm
        - style.size_mm / 2.0;
    let x = if plan.number % 2 == 1 {
        metrics.text_right() - metrics.char_pitch - width
    } else {
        metrics.text_left() + metrics.char_pitch
    };
    typeset::draw_left(pixmap, book, style, x, baseline, &text, black, transform)?;
    Ok(())
}

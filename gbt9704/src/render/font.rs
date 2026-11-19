//! 阶段 2 字体库: 字体文件发现、字形度量与字形轮廓绘制.
//!
//! 为什么需要这一层: GB/T 9704-2012 只规定字体族(小标宋/仿宋/黑体/楷体), 不规定字体文件;
//! `fonts/` 里的开源替代(映射见 `docs/fonts.md` 与 `fonts-manifest.json`)由 ttf-parser
//! 取出字形轮廓, 再交给 tiny-skia 填充. 文字、红分隔线、印章因此共享同一套光栅化,
//! 不需要额外的 PDF/HTML 排版器.
//!
//! 坐标约定: 本文件所有长度单位都是 mm, 原点在页面左上角, y 向下; 页面像素化由调用方
//! 用 `Transform` 在 `fill_path` 时完成, 这样排版逻辑与 DPI 无关.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use tiny_skia::{Color, FillRule, Paint, Path as SkiaPath, PathBuilder, Pixmap, Transform};
use ttf_parser::{Face, OutlineBuilder};

use crate::gbt9704::fonts::FontSlot;
use crate::gbt9704::geometry;

/// 汉字墨迹框相对基线的位置: 基线在 em 框顶部往下 0.88 em 处, 墨迹下沿约在 +0.12 em.
///
/// 为什么用常数而不用 `Face::ascender()`: Noto CJK 的 hhea ascender 是 1.16 em,
/// Fandol 是 0.88 em, 直接取字体度量会让同一行里不同字族的基线错开约 1.5 mm;
/// 汉字实际墨迹都落在 -0.88..+0.12 em, 用常数才能跨字族对齐.
pub const BASELINE_EM: f64 = 0.88;

/// 汉字墨迹下沿相对基线的位置(em).
pub const DESCENT_EM: f64 = 0.12;

/// 网格紧缩系数: 3 号字每行 28 字要求字宽 156/28 mm, 比名义 16 pt(5.6444 mm)窄约 1.3%.
///
/// 为什么所有字号乘同一个系数: 3 号正文必须正好落在 28 字网格上, 其他要素(标题/版头)
/// 保持与正文相同的紧缩比例, 版面才是同一套字距.
pub fn grid_condense() -> f64 {
    geometry::char_pitch_mm() / geometry::pt_to_mm(geometry::SAN_HAO_PT)
}

/// 字号(pt) 转 em 框高度(mm).
pub fn size_mm(pt: f64) -> f64 {
    geometry::pt_to_mm(pt)
}

/// 槽位 → 字体文件候选名(按优先级). 文件由 `scripts/fetch-fonts.sh` 落到 `fonts/`.
const SLOT_FILES: &[(FontSlot, &[&str])] = &[
    (
        FontSlot::Title,
        &["NotoSerifCJKsc-Bold.otf", "FandolSong-Bold.otf"],
    ),
    (
        FontSlot::IssuerMark,
        &["NotoSerifCJKsc-Bold.otf", "FandolSong-Bold.otf"],
    ),
    (
        FontSlot::Body,
        &["FandolFang-Regular.otf", "ZhuqueFangsong-Regular.ttf"],
    ),
    (
        FontSlot::Record,
        &["FandolFang-Regular.otf", "ZhuqueFangsong-Regular.ttf"],
    ),
    (
        FontSlot::Heiti,
        &["NotoSansCJKsc-Regular.otf", "FandolHei-Regular.otf"],
    ),
    (
        FontSlot::Kaiti,
        &["LXGWWenKai-Regular.ttf", "FandolKai-Regular.otf"],
    ),
    (
        FontSlot::PageNumber,
        &["NotoSerifCJKsc-Regular.otf", "FandolSong-Regular.otf"],
    ),
];

/// 打开的字体文件. 字节留在内存里, 每次取 `Face` 时现场解析(解析一张表头只需微秒级),
/// 避免自引用结构.
struct FontFile {
    path: PathBuf,
    data: Vec<u8>,
}

/// 全部字体槽位的字节集合.
pub struct FontBook {
    slots: HashMap<FontSlot, FontFile>,
}

impl FontBook {
    /// 按目录加载全部槽位; `None` 时依次尝试 `$GBT9704_FONTS`、`./fonts`、
    /// crate 根下的 `fonts/`(cargo run 在 workspace 根执行时 CWD 是 workspace).
    pub fn discover(dir: Option<&Path>) -> Result<Self, String> {
        let dir = match dir {
            Some(dir) => dir.to_path_buf(),
            None => default_fonts_dir(),
        };
        let mut slots = HashMap::new();
        for &(slot, candidates) in SLOT_FILES {
            let mut found = None;
            for name in candidates {
                let path = dir.join(name);
                match fs::read(&path) {
                    Ok(data) => {
                        found = Some(FontFile { path, data });
                        break;
                    }
                    Err(_) => continue,
                }
            }
            let Some(file) = found else {
                return Err(format!(
                    "字体目录 {} 缺少槽位 {} 的字体文件(候选: {:?}); 先运行 scripts/fetch-fonts.sh",
                    dir.display(),
                    slot.id(),
                    candidates
                ));
            };
            slots.insert(slot, file);
        }
        Ok(Self { slots })
    }

    /// 取槽位对应的字体面.
    pub fn face(&self, slot: FontSlot) -> Result<Face<'_>, String> {
        let file = self
            .slots
            .get(&slot)
            .ok_or_else(|| format!("字体槽位 {} 未加载", slot.id()))?;
        Face::parse(&file.data, 0)
            .map_err(|error| format!("解析字体 {} 失败: {error:?}", file.path.display()))
    }

    /// 已加载的字体文件路径, 供 CLI 打印来源.
    pub fn loaded(&self) -> Vec<(&'static str, PathBuf)> {
        SLOT_FILES
            .iter()
            .filter_map(|&(slot, _)| {
                self.slots
                    .get(&slot)
                    .map(|file| (slot.id(), file.path.clone()))
            })
            .collect()
    }
}

fn default_fonts_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("GBT9704_FONTS") {
        return PathBuf::from(dir);
    }
    let candidates = [
        PathBuf::from("fonts"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fonts"),
    ];
    for dir in candidates {
        if dir.join(SLOT_FILES[0].1[0]).exists() {
            return dir;
        }
    }
    PathBuf::from("fonts")
}

/// 一个字符的名义步进(mm); 缺字时按半角 0.5 em 估宽, 保证不因字体缺字而错位.
pub fn natural_advance_mm(face: &Face<'_>, size_mm: f64, ch: char) -> f64 {
    let upem = face.units_per_em() as f64;
    match face
        .glyph_index(ch)
        .and_then(|glyph| face.glyph_hor_advance(glyph))
    {
        Some(advance) => advance as f64 / upem * size_mm,
        None => 0.5 * size_mm,
    }
}

/// 公文网格步进: 自然步进 × 网格紧缩系数.
pub fn advance_mm(face: &Face<'_>, size_mm: f64, ch: char) -> f64 {
    natural_advance_mm(face, size_mm, ch) * grid_condense()
}

/// 一段文字在公文网格下的宽度(mm).
pub fn grid_width_mm(face: &Face<'_>, size_mm: f64, text: &str) -> f64 {
    text.chars().map(|ch| advance_mm(face, size_mm, ch)).sum()
}

/// 字形轮廓 → tiny-skia Path 的适配器.
struct GlyphSink<'a> {
    builder: &'a mut PathBuilder,
    scale: f32,
    origin_x: f32,
    origin_y: f32,
}

impl OutlineBuilder for GlyphSink<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.builder.move_to(
            self.origin_x + x * self.scale,
            self.origin_y - y * self.scale,
        );
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.builder.line_to(
            self.origin_x + x * self.scale,
            self.origin_y - y * self.scale,
        );
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.builder.quad_to(
            self.origin_x + x1 * self.scale,
            self.origin_y - y1 * self.scale,
            self.origin_x + x * self.scale,
            self.origin_y - y * self.scale,
        );
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.builder.cubic_to(
            self.origin_x + x1 * self.scale,
            self.origin_y - y1 * self.scale,
            self.origin_x + x2 * self.scale,
            self.origin_y - y2 * self.scale,
            self.origin_x + x * self.scale,
            self.origin_y - y * self.scale,
        );
    }

    fn close(&mut self) {
        self.builder.close();
    }
}

/// 单个字形轮廓(mm, 原点在基线左端, y 向下).
pub fn glyph_path(face: &Face<'_>, size_mm: f64, ch: char) -> Option<SkiaPath> {
    let glyph = face.glyph_index(ch)?;
    let scale = (size_mm / face.units_per_em() as f64) as f32;
    let mut builder = PathBuilder::new();
    let mut sink = GlyphSink {
        builder: &mut builder,
        scale,
        origin_x: 0.0,
        origin_y: 0.0,
    };
    face.outline_glyph(glyph, &mut sink)?;
    builder.finish()
}

/// 用纯色填充一条路径.
pub fn fill_path(pm: &mut Pixmap, path: &SkiaPath, color: Color, transform: Transform) {
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    pm.fill_path(path, &paint, FillRule::Winding, transform, None);
}

/// 用纯色描边一条路径.
pub fn stroke_path(
    pm: &mut Pixmap,
    path: &SkiaPath,
    width_mm: f64,
    color: Color,
    transform: Transform,
) {
    let mut paint = Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    let stroke = tiny_skia::Stroke {
        width: width_mm as f32,
        ..Default::default()
    };
    pm.stroke_path(path, &paint, &stroke, transform, None);
}

/// 在给定基线上画一行(或一段)文字, 返回画完后的笔位 x(mm).
///
/// `condensed` 为真时按 28 字网格紧缩(正文/标题等); 印章弧形文字用自然步进.
#[allow(clippy::too_many_arguments)]
pub fn draw_text(
    pm: &mut Pixmap,
    face: &Face<'_>,
    size_mm: f64,
    x_mm: f64,
    baseline_mm: f64,
    color: Color,
    text: &str,
    tracking_mm: f64,
    condensed: bool,
    transform: Transform,
) -> f64 {
    let scale = (size_mm / face.units_per_em() as f64) as f32;
    let mut builder = PathBuilder::new();
    let mut pen = x_mm;
    for ch in text.chars() {
        if let Some(glyph) = face.glyph_index(ch) {
            let mut sink = GlyphSink {
                builder: &mut builder,
                scale,
                origin_x: pen as f32,
                origin_y: baseline_mm as f32,
            };
            face.outline_glyph(glyph, &mut sink);
        }
        pen += if condensed {
            advance_mm(face, size_mm, ch)
        } else {
            natural_advance_mm(face, size_mm, ch)
        } + tracking_mm;
    }
    if let Some(path) = builder.finish() {
        fill_path(pm, &path, color, transform);
    }
    pen
}

/// 在 `transform` 下画单个字形(字形坐标原点在基线左端); 印章的旋转文字用.
pub fn draw_glyph(
    pm: &mut Pixmap,
    face: &Face<'_>,
    size_mm: f64,
    ch: char,
    color: Color,
    transform: Transform,
) -> Option<f64> {
    let glyph = face.glyph_index(ch)?;
    let scale = (size_mm / face.units_per_em() as f64) as f32;
    let mut builder = PathBuilder::new();
    let mut sink = GlyphSink {
        builder: &mut builder,
        scale,
        origin_x: 0.0,
        origin_y: 0.0,
    };
    face.outline_glyph(glyph, &mut sink)?;
    let path = builder.finish()?;
    fill_path(pm, &path, color, transform);
    Some(natural_advance_mm(face, size_mm, ch))
}

/// 十六进制颜色转 tiny-skia 颜色.
pub fn color(red: u8, green: u8, blue: u8, alpha: u8) -> Color {
    Color::from_rgba8(red, green, blue, alpha)
}

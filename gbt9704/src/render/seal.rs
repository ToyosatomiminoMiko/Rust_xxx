//! 圆形公章绘制: 外圆、环形机关名称、中央五角星、下方编号.
//!
//! 为什么自己画: GB/T 9704-2012 只规定"印章用红色, 端正、居中下压发文机关署名和成文日期"
//! (7.3.5.1), 公章图形本身由印章管理规定; 输入 JSON 里的印章可能只给了机关名称而没有
//! 图片, 这里按公章版式现场生成, 保证"实施效果"不依赖外部资源.
//!
//! 版式依据(圆形机关印章的通行样式): 外圆线宽约为直径的 2%, 机关名称沿上半圆环排列,
//! 字头朝外、按弧长均匀分布; 五角星居中, 外接圆半径约为直径的 19%; 编号在下方居中.

use tiny_skia::{Pixmap, Transform};

use crate::gbt9704::fonts::FontSlot;

use super::draw;
use super::font::{self, FontBook};

/// 一枚要绘制的印章.
#[derive(Clone, Debug, Default)]
pub struct SealSpec {
    /// 环形文字(通常是发文机关全称).
    pub text: String,
    /// 印章编号; 无则省略.
    pub number: Option<String>,
}

/// 在 `(center_x, center_y)` 处画一枚直径 `diameter_mm` 的公章.
pub fn draw(
    pm: &mut Pixmap,
    book: &FontBook,
    center_x: f64,
    center_y: f64,
    diameter_mm: f64,
    spec: &SealSpec,
    transform: Transform,
) -> Result<(), String> {
    let radius = diameter_mm / 2.0;
    // 印泥是半透明油墨: 压住署名和日期时, 下面的黑字仍然透出来.
    let base = super::ink_red();
    let ink = font::color(
        (base.red() * 255.0).round() as u8,
        (base.green() * 255.0).round() as u8,
        (base.blue() * 255.0).round() as u8,
        226,
    );

    let ring_width = (diameter_mm * 0.021).max(0.6);
    draw::stroke_circle(
        pm,
        center_x,
        center_y,
        radius - ring_width / 2.0,
        ring_width,
        ink,
        transform,
    );

    // 中央五角星.
    let star_outer = radius * 0.36;
    let star_inner = star_outer * 0.382;
    let points = draw::star_points(center_x, center_y, star_outer, star_inner);
    draw::fill_polygon(pm, &points, ink, transform);

    // 环形机关名称.
    let face = book.face(FontSlot::Title)?;
    draw_arc_text(
        pm,
        &face,
        center_x,
        center_y,
        radius,
        ring_width,
        diameter_mm,
        &spec.text,
        ink,
        transform,
    )?;

    // 编号.
    if let Some(number) = &spec.number {
        let size = diameter_mm * 0.062;
        let width = number
            .chars()
            .map(|ch| font::natural_advance_mm(&face, size, ch))
            .sum::<f64>();
        let baseline = center_y + radius * 0.70;
        font::draw_text(
            pm,
            &face,
            size,
            center_x - width / 2.0,
            baseline,
            ink,
            number,
            0.0,
            false,
            transform,
        );
    }
    Ok(())
}

/// 沿上半圆环排字: 每个字旋转到"字头朝外、字底朝圆心".
#[allow(clippy::too_many_arguments)]
fn draw_arc_text(
    pm: &mut Pixmap,
    face: &ttf_parser::Face<'_>,
    center_x: f64,
    center_y: f64,
    radius: f64,
    ring_width: f64,
    diameter_mm: f64,
    text: &str,
    ink: tiny_skia::Color,
    transform: Transform,
) -> Result<(), String> {
    let chars: Vec<char> = text.chars().filter(|ch| !ch.is_whitespace()).collect();
    if chars.is_empty() {
        return Ok(());
    }
    let size = diameter_mm * 0.112;
    // 字身内沿离圆环内边留一点空隙, 基线圆半径由此反推.
    let baseline_radius = radius - ring_width - diameter_mm * 0.016 - font::BASELINE_EM * size;
    let advance = font::natural_advance_mm(face, size, chars[0]);
    // 按弧长均匀排布; 名称越长跨度越大, 但不超过 250°, 否则首尾字会挤到下半圆.
    let max_span = 238f64.to_radians();
    let step = (advance * 0.99 / baseline_radius).min(max_span / (chars.len() - 1).max(1) as f64);
    let start = -std::f64::consts::FRAC_PI_2 - (chars.len().saturating_sub(1)) as f64 * step / 2.0;

    for (index, &ch) in chars.iter().enumerate() {
        let angle = start + index as f64 * step;
        let (sin, cos) = (angle.sin(), angle.cos());
        let px = center_x + baseline_radius * cos;
        let py = center_y + baseline_radius * sin;
        let char_advance = font::natural_advance_mm(face, size, ch) / 2.0;
        // 基线方向 = 圆弧切线, 字头方向 = 背离圆心; 行列式必须为 +1(纯旋转, 不镜像).
        let char_transform = Transform::from_row(
            -sin as f32,
            cos as f32,
            -cos as f32,
            -sin as f32,
            (px + sin * char_advance) as f32,
            (py - cos * char_advance) as f32,
        )
        .post_concat(transform);
        font::draw_glyph(pm, face, size, ch, ink, char_transform);
    }
    Ok(())
}

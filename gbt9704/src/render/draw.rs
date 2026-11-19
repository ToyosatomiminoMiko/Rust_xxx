//! 基础图形: 直线、圆周、圆、多边形与五角星.
//!
//! 为什么单独一层: 红色分隔线、版记线、印章轮廓与五角星需要和文字共用同一套
//! mm 坐标与 `Transform`, 集中在这里可以避免每个调用点重复写 PathBuilder.

use tiny_skia::{Color, PathBuilder, Pixmap, Transform};

/// 画一条直线(描边, 线宽为 `width_mm`).
pub fn line(
    pm: &mut Pixmap,
    x1: f64,
    y1: f64,
    x2: f64,
    y2: f64,
    width_mm: f64,
    color: Color,
    transform: Transform,
) {
    let mut builder = PathBuilder::new();
    builder.move_to(x1 as f32, y1 as f32);
    builder.line_to(x2 as f32, y2 as f32);
    if let Some(path) = builder.finish() {
        super::font::stroke_path(pm, &path, width_mm, color, transform);
    }
}

/// 画一条粗直线(用矩形填充, 避免描边在端点处圆角/羽化差异).
pub fn bar(
    pm: &mut Pixmap,
    x1: f64,
    y1: f64,
    x2: f64,
    thickness_mm: f64,
    color: Color,
    transform: Transform,
) {
    let mut builder = PathBuilder::new();
    let top = y1 - thickness_mm / 2.0;
    let bottom = y1 + thickness_mm / 2.0;
    builder.move_to(x1 as f32, top as f32);
    builder.line_to(x2 as f32, top as f32);
    builder.line_to(x2 as f32, bottom as f32);
    builder.line_to(x1 as f32, bottom as f32);
    builder.close();
    if let Some(path) = builder.finish() {
        super::font::fill_path(pm, &path, color, transform);
    }
}

/// 描边圆周(线宽中心落在半径 r 上).
pub fn stroke_circle(
    pm: &mut Pixmap,
    cx: f64,
    cy: f64,
    r: f64,
    width_mm: f64,
    color: Color,
    transform: Transform,
) {
    let mut builder = PathBuilder::new();
    builder.push_circle(cx as f32, cy as f32, r as f32);
    if let Some(path) = builder.finish() {
        super::font::stroke_path(pm, &path, width_mm, color, transform);
    }
}

/// 填充多边形.
pub fn fill_polygon(
    pm: &mut Pixmap,
    points: &[(f64, f64)],
    color: Color,
    transform: Transform,
) {
    let Some((&(x0, y0), rest)) = points.split_first() else {
        return;
    };
    let mut builder = PathBuilder::new();
    builder.move_to(x0 as f32, y0 as f32);
    for &(x, y) in rest {
        builder.line_to(x as f32, y as f32);
    }
    builder.close();
    if let Some(path) = builder.finish() {
        super::font::fill_path(pm, &path, color, transform);
    }
}

/// 五角星顶点: 外接圆半径 `outer`、内接圆半径 `inner`, 一个尖角朝正上方.
///
/// 为什么单独算顶点: 印章中央的五角星是公章法定图形要素, 不能拿字体里的 "★" 代替
/// (字形风格/比例与公章不一致).
pub fn star_points(cx: f64, cy: f64, outer: f64, inner: f64) -> Vec<(f64, f64)> {
    let mut points = Vec::with_capacity(10);
    for index in 0..10 {
        let radius = if index % 2 == 0 { outer } else { inner };
        // 从正上方(-90°)开始顺时针.
        let angle = -std::f64::consts::FRAC_PI_2 + index as f64 * std::f64::consts::PI / 5.0;
        points.push((cx + radius * angle.cos(), cy + radius * angle.sin()));
    }
    points
}

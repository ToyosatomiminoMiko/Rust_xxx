//! GB/T 9704-2012 的版面几何: 纸张、页边、版心、网格、字号与颜色.
//!
//! 来源: GB/T 9704-2012 第 4、5 章与 3.1/3.2 术语定义.
//!
//! 关键换算:
//! - 1 英寸 = 25.4 mm = 72 pt, 故 `pt = mm * 72 / 25.4`;
//! - 版心 156 mm x 225 mm, 每面 22 行 x 每行 28 字;
//! - 网格字距 = 156 / 28 mm ≈ 5.5714 mm ≈ 15.79 pt (3 号字名义 16 pt);
//! - 网格行距 = 225 / 22 mm ≈ 10.2273 mm ≈ 28.99 pt.
//!
//! 坑: 标准 3.2 把"行"定义为"一个汉字的高度加 3 号汉字高度的 7/8 的距离",
//! 3 号按 16 pt 计算得出 30 pt; 而 22 行撑满 225 mm 要求 28.99 pt, 两者相差约 1 pt.
//! 本模块同时提供 `line_unit_pt()` (术语定义值, 用于"空一行"等文本间距的语义说明)
//! 与 `line_pitch_pt()` (版心网格值, 用于真实排版), 调用方必须明确选哪一个.

use std::fmt;

/// 长度单位换算用常数: 每英寸 25.4 毫米.
pub const MM_PER_INCH: f64 = 25.4;
/// 长度单位换算用常数: 每英寸 72 点.
pub const PT_PER_INCH: f64 = 72.0;

/// 毫米转点 (PostScript point, 1/72 inch).
pub fn mm_to_pt(mm: f64) -> f64 {
    mm * PT_PER_INCH / MM_PER_INCH
}

/// 点转毫米.
pub fn pt_to_mm(pt: f64) -> f64 {
    pt * MM_PER_INCH / PT_PER_INCH
}

// ---------------------------------------------------------------------------
// 5.1 幅面尺寸 / 5.2.1 页边与版心
// ---------------------------------------------------------------------------

/// A4 成品幅面宽 (GB/T 148 的 A4): 210 mm.
pub const PAPER_WIDTH_MM: f64 = 210.0;
/// A4 成品幅面高: 297 mm.
pub const PAPER_HEIGHT_MM: f64 = 297.0;
/// 天头(上白边): 37 mm ± 1 mm.
pub const MARGIN_TOP_MM: f64 = 37.0;
/// 订口(左白边): 28 mm ± 1 mm.
pub const MARGIN_LEFT_MM: f64 = 28.0;
/// 版心宽: 156 mm.
pub const TEXT_WIDTH_MM: f64 = 156.0;
/// 版心高: 225 mm.
pub const TEXT_HEIGHT_MM: f64 = 225.0;

/// 右侧白边 = 210 - 28 - 156 = 26 mm (标准未直接给出, 由幅面与版心推导).
pub const MARGIN_RIGHT_MM: f64 = PAPER_WIDTH_MM - MARGIN_LEFT_MM - TEXT_WIDTH_MM;
/// 下白边 = 297 - 37 - 225 = 35 mm (标准未直接给出, 由幅面与版心推导).
pub const MARGIN_BOTTOM_MM: f64 = PAPER_HEIGHT_MM - MARGIN_TOP_MM - TEXT_HEIGHT_MM;

// ---------------------------------------------------------------------------
// 5.2.3 行数和字数
// ---------------------------------------------------------------------------

/// 每面行数: 22.
pub const LINES_PER_PAGE: u32 = 22;
/// 每行字数: 28.
pub const CHARS_PER_LINE: u32 = 28;

/// 一个汉字的网格宽度(版心宽 / 每行字数), 单位 mm.
pub fn char_pitch_mm() -> f64 {
    TEXT_WIDTH_MM / CHARS_PER_LINE as f64
}

/// 一个汉字的网格宽度, 单位 pt.
pub fn char_pitch_pt() -> f64 {
    mm_to_pt(char_pitch_mm())
}

/// 网格行距(版心高 / 每面行数), 单位 mm.
pub fn line_pitch_mm() -> f64 {
    TEXT_HEIGHT_MM / LINES_PER_PAGE as f64
}

/// 网格行距, 单位 pt.
pub fn line_pitch_pt() -> f64 {
    mm_to_pt(line_pitch_mm())
}

// ---------------------------------------------------------------------------
// 3.1 / 3.2 术语: 字、行
// ---------------------------------------------------------------------------

/// 3 号字的名义字号: 16 pt.
pub const SAN_HAO_PT: f64 = 16.0;

/// 术语"一行"的长度: 一个汉字高度 + 3 号汉字高度的 7/8 = 16 + 14 = 30 pt.
///
/// 注意: 这是 3.2 的术语定义值, 不等于实际网格行距 `line_pitch_pt()`.
pub fn line_unit_pt() -> f64 {
    SAN_HAO_PT + SAN_HAO_PT * 7.0 / 8.0
}

/// 术语"一行"的长度, 单位 mm.
pub fn line_unit_mm() -> f64 {
    pt_to_mm(line_unit_pt())
}

/// "空 N 行"按网格行距换算成垂直距离(mm).
pub fn blank_lines_mm(lines: u32) -> f64 {
    lines as f64 * line_pitch_mm()
}

// ---------------------------------------------------------------------------
// 中文字号表 (字号 -> pt)
// ---------------------------------------------------------------------------

/// 常用中文字号与点值. 标准用到的: 2 号 22 pt, 3 号 16 pt, 4 号 14 pt.
pub const FONT_SIZES: &[(&str, f64)] = &[
    ("初号", 42.0),
    ("小初", 36.0),
    ("一号", 26.0),
    ("小一", 24.0),
    ("二号", 22.0),
    ("小二", 18.0),
    ("三号", 16.0),
    ("小三", 15.0),
    ("四号", 14.0),
    ("小四", 12.0),
    ("五号", 10.5),
    ("小五", 9.0),
];

/// 按中文字号取点值; 未知字号返回 None.
pub fn chinese_font_size_pt(name: &str) -> Option<f64> {
    FONT_SIZES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, pt)| *pt)
}

// ---------------------------------------------------------------------------
// 版头 / 版记 / 页码 / 特定格式的定位常数
// ---------------------------------------------------------------------------

/// 7.2.4 发文机关标志上边缘至版心上边缘: 35 mm.
pub const ISSUER_MARK_TOP_FROM_TEXT_MM: f64 = 35.0;
/// 10.2 命令(令) 发文机关标志上边缘至版心上边缘: 20 mm.
pub const ORDER_MARK_TOP_FROM_TEXT_MM: f64 = 20.0;
/// 10.1 信函格式发文机关标志上边缘至上页边: 30 mm.
pub const LETTER_MARK_TOP_FROM_PAPER_MM: f64 = 30.0;
/// 10.1 信函格式第一条红色双线距发文机关标志: 4 mm.
pub const LETTER_RED_LINE_TOP_GAP_MM: f64 = 4.0;
/// 10.1 信函格式第二条红色双线距下页边: 20 mm.
pub const LETTER_RED_LINE_BOTTOM_GAP_MM: f64 = 20.0;
/// 10.1 信函格式红色双线长度: 170 mm.
pub const LETTER_RED_LINE_WIDTH_MM: f64 = 170.0;
/// 7.2.7 发文字号之下的红色分隔线间距: 4 mm.
pub const HEADER_RED_LINE_GAP_MM: f64 = 4.0;

/// 7.4.1 版记首条/末条分隔线推荐高度: 0.35 mm.
pub const RECORD_LINE_THICK_MM: f64 = 0.35;
/// 7.4.1 版记中间分隔线推荐高度: 0.25 mm.
pub const RECORD_LINE_THIN_MM: f64 = 0.25;

/// 7.5 页码一字线上距版心下边缘: 7 mm.
pub const PAGE_NUMBER_LINE_OFFSET_MM: f64 = 7.0;
/// 7.5 页码在单/双页分别与版心边缘的空字距: 1 字.
pub const PAGE_NUMBER_SIDE_BLANK_CHARS: u32 = 1;
/// 7.5 页码数字用 4 号宋体: 14 pt.
pub const PAGE_NUMBER_FONT_PT: f64 = 14.0;
/// 7.5 页码"一字线"长度标准未定义; 建议取 4 号字宽(14 pt ≈ 4.9389 mm), 图测≈4.6 mm, 可配置.
pub const PAGE_NUMBER_DASH_WIDTH_MM: f64 = 4.938_9;

// ---------------------------------------------------------------------------
// 标准未规定、需要实现给默认值的地方(来自式样图测量/字体调研, 非规范值)
// ---------------------------------------------------------------------------

/// 7.2.4 发文机关标志字号标准未规定; 建议 22–48 pt 自适应, 默认约 36 pt(图测 33–39 pt).
pub const ISSUER_MARK_DEFAULT_PT: f64 = 36.0;
/// 发文机关标志字号建议下限.
pub const ISSUER_MARK_MIN_PT: f64 = 22.0;
/// 发文机关标志字号建议上限(单行不超过版心 156 mm 时可按名称长度收缩).
pub const ISSUER_MARK_MAX_PT: f64 = 48.0;
/// 7.2.7 红色分隔线厚度标准未规定; 式样图图测≈0.6 mm, 仅作渲染默认提示.
pub const HEADER_RED_LINE_THICKNESS_HINT_MM: f64 = 0.6;
/// 名义 3 号字宽; 注意 28 字网格步进用 `char_pitch_pt()`(≈15.79 pt), 不是 16 pt.
pub const CHAR_NOMINAL_PT: f64 = SAN_HAO_PT;

// ---------------------------------------------------------------------------
// 5.2.4 文字颜色 / 6.2 红色油墨
// ---------------------------------------------------------------------------

/// RGB 颜色. 打印/PDF 阶段应优先使用 6.2 的 CMYK 油墨值.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// 输出 `#RRGGBB` 形式, 便于写入 CSS/SVG/PDF 元数据.
    pub fn hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hex())
    }
}

/// 5.2.4 如无特殊说明, 公文中文字颜色均为黑色.
pub const BLACK: Color = Color::new(0x00, 0x00, 0x00);

/// 6.2 红色油墨: 色谱 Y80%、M80% (C=0, K=0) 的近似 sRGB 值.
///
/// 坑: 屏幕 RGB 与印刷 CMYK 不是一一对应, 这里用朴素换算: R=255, G=B=255*(1-0.8)=51.
/// 真正印刷/CMYK PDF 应使用 0/0.8/0.8/0, 而不是这个 RGB 值.
pub const RED: Color = Color::new(0xFF, 0x33, 0x33);

/// 6.2 红色油墨的 CMYK 值 (C, M, Y, K), 范围 0.0~1.0.
pub const RED_CMYK: (f64, f64, f64, f64) = (0.0, 0.8, 0.8, 0.0);

/// 6.2 黑色油墨: 色谱 BL100%, 即 K=1.
pub const BLACK_INK: (u8, u8, u8) = (0, 0, 0);

// ---------------------------------------------------------------------------
// 4 公文用纸主要技术指标
// ---------------------------------------------------------------------------

/// 4 纸张定量范围 (g/m²).
pub const PAPER_WEIGHT_G_PER_M2: (u32, u32) = (60, 80);
/// 4 纸张白度范围 (%).
pub const PAPER_WHITENESS_PERCENT: (u32, u32) = (80, 90);
/// 4 横向耐折度要求: ≥15 次.
pub const PAPER_FOLD_ENDURANCE_MIN: u32 = 15;
/// 4 不透明度要求: ≥85%.
pub const PAPER_OPACITY_MIN_PERCENT: u32 = 85;
/// 4 pH 值范围.
pub const PAPER_PH_RANGE: (f64, f64) = (7.5, 9.5);

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn margins_are_derived_from_paper_and_text_area() {
        assert!(close(MARGIN_RIGHT_MM, 26.0));
        assert!(close(MARGIN_BOTTOM_MM, 35.0));
        assert!(close(
            MARGIN_LEFT_MM + TEXT_WIDTH_MM + MARGIN_RIGHT_MM,
            PAPER_WIDTH_MM
        ));
        assert!(close(
            MARGIN_TOP_MM + TEXT_HEIGHT_MM + MARGIN_BOTTOM_MM,
            PAPER_HEIGHT_MM
        ));
    }

    #[test]
    fn grid_pitch_matches_standard_mm() {
        assert!(close(char_pitch_mm() * 28.0, 156.0));
        assert!(close(line_pitch_mm() * 22.0, 225.0));
        assert!(close(char_pitch_pt(), 156.0 * 72.0 / (28.0 * 25.4)));
        assert!(close(line_pitch_pt(), 225.0 * 72.0 / (22.0 * 25.4)));
    }

    #[test]
    fn line_unit_follows_definition_not_grid() {
        // 3.2 定义: 16 + 16*7/8 = 30 pt, 与网格行距 28.99 pt 不同.
        assert!(close(line_unit_pt(), 30.0));
        assert!((line_unit_pt() - line_pitch_pt()).abs() > 0.9);
    }

    #[test]
    fn font_size_table_contains_standard_sizes() {
        assert_eq!(chinese_font_size_pt("二号"), Some(22.0));
        assert_eq!(chinese_font_size_pt("三号"), Some(16.0));
        assert_eq!(chinese_font_size_pt("四号"), Some(14.0));
        assert_eq!(chinese_font_size_pt("五号"), Some(10.5));
        assert_eq!(chinese_font_size_pt("不存在"), None);
    }

    #[test]
    fn unregulated_defaults_are_consistent_with_measurements() {
        assert!((PAGE_NUMBER_DASH_WIDTH_MM - pt_to_mm(14.0)).abs() < 0.001);
        let (min_pt, default_pt, max_pt) = (
            ISSUER_MARK_MIN_PT,
            ISSUER_MARK_DEFAULT_PT,
            ISSUER_MARK_MAX_PT,
        );
        assert!(min_pt < default_pt);
        assert!(default_pt < max_pt);
        // 28 个名义 16 pt 汉字 = 158.0444 mm, 比版心 156 mm 宽约 2.0444 mm; 网格步进必须用 156/28.
        assert!((28.0 * pt_to_mm(CHAR_NOMINAL_PT) - TEXT_WIDTH_MM - 2.0444).abs() < 0.001);
        assert!(close(28.0 * char_pitch_mm(), TEXT_WIDTH_MM));
    }

    #[test]
    fn colors_render_as_hex() {
        assert_eq!(BLACK.hex(), "#000000");
        assert_eq!(RED.hex(), "#FF3333");
    }

    #[test]
    fn mm_pt_conversion_is_symmetric() {
        for mm in [1.0, 4.0, 7.0, 25.4, 156.0, 225.0, 297.0] {
            assert!(close(pt_to_mm(mm_to_pt(mm)), mm));
        }
    }
}

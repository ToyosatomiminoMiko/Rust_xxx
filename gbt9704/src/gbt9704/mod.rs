//! GB/T 9704-2012《党政机关公文格式》的机器可读模型.
//!
//! 子模块分工:
//! - `geometry`: 纸张/版心/网格/字号/颜色/定位常数与换算;
//! - `fonts`: 字体槽位、标准字体要求与开源/商业字体候选;
//! - `elements`: 全部格式要素规格(及标准条款号), 供映射与文档生成使用.
//!
//! 标准原文只在此模块之外被引用(见各条的 `clause` 字段), 渲染与生成逻辑应只依赖这里的结构化数据.

pub mod elements;
pub mod fonts;
pub mod geometry;

pub use elements::{
    Alignment, ELEMENTS, ElementSpec, Part, Requirement, by_part, find as find_element,
};
pub use fonts::{CANDIDATES, FontCandidate, FontRequirement, FontSlot, REQUIREMENTS};
pub use geometry::{
    BLACK, Color, MM_PER_INCH, PAPER_HEIGHT_MM, PAPER_WIDTH_MM, PT_PER_INCH, RED, RED_CMYK,
    char_pitch_mm, char_pitch_pt, chinese_font_size_pt, line_pitch_mm, line_pitch_pt, line_unit_mm,
    line_unit_pt, mm_to_pt, pt_to_mm,
};

/// 标准标识, 写入 AKN 的呈现/扩展元数据.
pub const STANDARD_ID: &str = "GB/T 9704-2012";
/// 标准中文全名.
pub const STANDARD_NAME: &str = "党政机关公文格式";
/// 标准发布日期.
pub const STANDARD_ISSUED: &str = "2012-06-29";
/// 标准实施日期.
pub const STANDARD_EFFECTIVE: &str = "2012-07-01";

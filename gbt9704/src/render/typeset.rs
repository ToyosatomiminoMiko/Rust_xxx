//! 正文排版: 28 字网格折行、行首/行尾禁则、两端对齐.
//!
//! 为什么自己折行: 公文正文必须"每行 28 字、回行顶格、撑满版心"(5.2.3/7.3.3),
//! 通用排版库的断行按词不按字, 会破坏这个网格; 这里按字推进, 字宽用
//! `font::advance_mm`(含网格紧缩), 折行点再受"禁则"约束.

use crate::gbt9704::fonts::FontSlot;

use super::font::{self, FontBook};

/// 行内文字样式: 字族槽位 + 字号(mm).
#[derive(Clone, Copy, Debug)]
pub struct TextStyle {
    pub slot: FontSlot,
    pub size_mm: f64,
}

impl TextStyle {
    pub fn new(slot: FontSlot, size_pt: f64) -> Self {
        Self {
            slot,
            size_mm: font::size_mm(size_pt),
        }
    }
}

/// 折行结果中的一行.
#[derive(Clone, Debug)]
pub struct Line {
    pub text: String,
    /// 文字本身宽度(mm), 不含首行缩进.
    pub width_mm: f64,
    /// 首行缩进(mm).
    pub indent_mm: f64,
    /// 是否为该自然地段的最后一行(最后一行不参与两端对齐).
    pub last: bool,
}

/// 折行时的禁则: 不能出现在行首的字符(收尾标点/后括号).
const NO_LINE_START: &[char] = &[
    '，', '。', '、', '；', '：', '？', '！', '）', '〕', '】', '》', '」', '』', '”', '’', '…',
    '—', '·', '％', '‰', ',', '.', ';', ':', '?', '!', ')', ']', '}', '>', '%',
];

/// 折行时的禁则: 不能出现在行尾的字符(起始标点/前括号).
const NO_LINE_END: &[char] = &[
    '（', '〔', '【', '《', '「', '『', '“', '‘', '(', '[', '{', '<',
];

fn no_line_start(ch: char) -> bool {
    NO_LINE_START.contains(&ch)
}

fn no_line_end(ch: char) -> bool {
    NO_LINE_END.contains(&ch)
}

/// ASCII 字母/数字连续体不拆行, 例如 "Faker"、"2013".
fn ascii_word(ch: char) -> bool {
    ch.is_ascii_alphanumeric()
}

/// 在 `max_width_mm` 内把段落折成若干行.
///
/// - `first_indent_mm`: 首行缩进(公文正文是 2 字);
/// - 行尾若是收尾标点且只溢出不到 0.65 字, 允许"悬挂"到版心外(标点悬挂),
///   否则退回到上一个可断点.
pub fn wrap(
    book: &FontBook,
    style: TextStyle,
    text: &str,
    max_width_mm: f64,
    first_indent_mm: f64,
) -> Result<Vec<Line>, String> {
    let face = book.face(style.slot)?;
    let chars: Vec<(char, f64)> = text
        .chars()
        .map(|ch| (ch, font::advance_mm(&face, style.size_mm, ch)))
        .collect();
    if chars.is_empty() {
        return Ok(Vec::new());
    }

    let mut lines = Vec::new();
    let mut start = 0usize;
    while start < chars.len() {
        let indent = if lines.is_empty() { first_indent_mm } else { 0.0 };
        let mut width = indent;
        let mut index = start;
        let mut last_break: Option<(usize, f64)> = None;
        let mut overflowed = false;

        while index < chars.len() {
            let (ch, advance) = chars[index];
            let excess = width + advance - max_width_mm;
            if excess > 1e-9 {
                // 收尾标点可以悬挂到版心外, 但不能超过半个字.
                if no_line_start(ch) && excess <= style.size_mm * 0.65 {
                    width += advance;
                    index += 1;
                    continue;
                }
                overflowed = true;
                break;
            }
            if index > start {
                let previous = chars[index - 1].0;
                let breakable = !no_line_start(ch)
                    && !no_line_end(previous)
                    && !(ascii_word(ch) && ascii_word(previous));
                if breakable {
                    last_break = Some((index, width));
                }
            }
            width += advance;
            index += 1;
        }

        if overflowed
            && let Some((break_index, break_width)) = last_break
            && break_index > start
        {
            index = break_index;
            width = break_width;
        }
        if index == start {
            // 单字超过整行宽(极端情况): 至少推进一个字, 避免死循环.
            width += chars[index].1;
            index += 1;
        }

        let mut end = index;
        while end > start && chars[end - 1].0 == ' ' {
            width -= chars[end - 1].1;
            end -= 1;
        }
        let line_text: String = chars[start..end].iter().map(|(ch, _)| *ch).collect();
        let last = index >= chars.len();
        lines.push(Line {
            text: line_text,
            width_mm: (width - indent).max(0.0),
            indent_mm: indent,
            last,
        });
        start = index;
    }
    Ok(lines)
}

/// 一段文字在网格紧缩下的宽度(mm), 用于居中/右对齐.
pub fn measure(book: &FontBook, style: TextStyle, text: &str) -> Result<f64, String> {
    let face = book.face(style.slot)?;
    Ok(font::grid_width_mm(&face, style.size_mm, text))
}

/// 在基线上画一行; `justify_to` 给出时把这一行两端对齐到该宽度(mm).
#[allow(clippy::too_many_arguments)]
pub fn draw_line(
    pm: &mut tiny_skia::Pixmap,
    book: &FontBook,
    style: TextStyle,
    x_mm: f64,
    baseline_mm: f64,
    line: &Line,
    color: tiny_skia::Color,
    justify_to: Option<f64>,
    transform: tiny_skia::Transform,
) -> Result<(), String> {
    let face = book.face(style.slot)?;
    let tracking = match justify_to {
        Some(target) if !line.last => {
            let gaps = line.text.chars().count().saturating_sub(1);
            if gaps == 0 {
                0.0
            } else {
                let tracking = (target - line.indent_mm - line.width_mm) / gaps as f64;
                // 只有剩余空间小于半个字才均摊, 否则宁可左对齐, 避免离谱的字距.
                if tracking > 0.0 && tracking <= style.size_mm * 0.5 {
                    tracking
                } else {
                    0.0
                }
            }
        }
        _ => 0.0,
    };
    font::draw_text(
        pm,
        &face,
        style.size_mm,
        x_mm + line.indent_mm,
        baseline_mm,
        color,
        &line.text,
        tracking,
        true,
        transform,
    );
    Ok(())
}

/// 画一段居中文字, `center_x_mm` 是版心中心(或任意对齐中心).
pub fn draw_centered(
    pm: &mut tiny_skia::Pixmap,
    book: &FontBook,
    style: TextStyle,
    center_x_mm: f64,
    baseline_mm: f64,
    text: &str,
    color: tiny_skia::Color,
    transform: tiny_skia::Transform,
) -> Result<(), String> {
    let face = book.face(style.slot)?;
    let width = font::grid_width_mm(&face, style.size_mm, text);
    font::draw_text(
        pm,
        &face,
        style.size_mm,
        center_x_mm - width / 2.0,
        baseline_mm,
        color,
        text,
        0.0,
        true,
        transform,
    );
    Ok(())
}

/// 画一段左对齐文字, 返回结束笔位(mm).
pub fn draw_left(
    pm: &mut tiny_skia::Pixmap,
    book: &FontBook,
    style: TextStyle,
    x_mm: f64,
    baseline_mm: f64,
    text: &str,
    color: tiny_skia::Color,
    transform: tiny_skia::Transform,
) -> Result<f64, String> {
    let face = book.face(style.slot)?;
    Ok(font::draw_text(
        pm,
        &face,
        style.size_mm,
        x_mm,
        baseline_mm,
        color,
        text,
        0.0,
        true,
        transform,
    ))
}

/// 基线位置: 让 em 框在一个字高为 `cell_h_mm` 的网格单元里垂直居中.
pub fn baseline_in_cell(cell_top_mm: f64, cell_h_mm: f64, size_mm: f64) -> f64 {
    cell_top_mm + (cell_h_mm - size_mm) / 2.0 + font::BASELINE_EM * size_mm
}

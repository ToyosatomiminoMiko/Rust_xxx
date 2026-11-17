//! 极简 XML 序列化工具.
//!
//! 为什么手写: 生成器只需要"写"XML, 复用标准库即可, 不引入 quick-xml/serde.
//! 本模块只保证: 合法转义、稳定的缩进、属性顺序可控.
//!
//! 坑:
//! - 属性值必须转义 `& < > "`, 以及 XML 1.0 不允许的控制字符;
//! - 文本节点中的 `>` 不是必须转义, 但 `]]>` 会破坏 CDATA, 这里统一转义 `>` 更安全;
//! - 混合内容(如 `<p>文字<docTitle>标题</docTitle></p>`)必须用 `*_inline` 系列函数,
//!   否则缩进会引入额外空白, 影响文本提取与渲染.

/// 转义 XML 文本节点.
pub fn escape_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            // XML 1.0 合法字符: #x9 | #xA | #xD | [#x20-...]
            '\u{0009}' | '\u{000A}' | '\u{000D}' => out.push(c),
            c if (c as u32) < 0x20 => out.push_str(&format!("&#x{:X};", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// 转义 XML 属性值(双引号包裹).
pub fn escape_attr(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            // 属性值中的换行/制表会被 XML 解析器规范化, 用字符引用保留.
            '\n' => out.push_str("&#10;"),
            '\r' => out.push_str("&#13;"),
            '\t' => out.push_str("&#9;"),
            c if (c as u32) < 0x20 => out.push_str(&format!("&#x{:X};", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// 一个缩进友好的 XML 写入器.
pub struct XmlWriter {
    out: String,
    depth: usize,
}

impl XmlWriter {
    pub fn new() -> Self {
        Self {
            out: String::new(),
            depth: 0,
        }
    }

    /// 写 XML 声明.
    pub fn declaration(&mut self) {
        self.out
            .push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    }

    fn indent(&mut self) {
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
    }

    fn open_tag(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.out.push('<');
        self.out.push_str(name);
        for (key, value) in attrs {
            self.out.push(' ');
            self.out.push_str(key);
            self.out.push_str("=\"");
            self.out.push_str(&escape_attr(value));
            self.out.push('"');
        }
    }

    /// 开始一个块级元素(带换行).
    pub fn start(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.indent();
        self.open_tag(name, attrs);
        self.out.push_str(">\n");
        self.depth += 1;
    }

    /// 结束块级元素.
    pub fn end(&mut self, name: &str) {
        self.depth = self.depth.saturating_sub(1);
        self.indent();
        self.out.push_str("</");
        self.out.push_str(name);
        self.out.push_str(">\n");
    }

    /// 空元素.
    pub fn empty(&mut self, name: &str, attrs: &[(&str, &str)]) {
        self.indent();
        self.open_tag(name, attrs);
        self.out.push_str("/>\n");
    }

    /// 只有文本内容的块级元素.
    pub fn text_element(&mut self, name: &str, attrs: &[(&str, &str)], text: &str) {
        self.indent();
        self.open_tag(name, attrs);
        self.out.push('>');
        self.out.push_str(&escape_text(text));
        self.out.push_str("</");
        self.out.push_str(name);
        self.out.push_str(">\n");
    }

    /// 内容已是 XML 片段的块级元素(用于混合内容).
    pub fn raw_element(&mut self, name: &str, attrs: &[(&str, &str)], inner_xml: &str) {
        self.indent();
        self.open_tag(name, attrs);
        self.out.push('>');
        self.out.push_str(inner_xml);
        self.out.push_str("</");
        self.out.push_str(name);
        self.out.push_str(">\n");
    }

    /// 追加一行注释.
    pub fn comment(&mut self, text: &str) {
        self.indent();
        self.out.push_str("<!-- ");
        self.out.push_str(&text.replace("--", "- -"));
        self.out.push_str(" -->\n");
    }

    pub fn finish(self) -> String {
        self.out
    }
}

impl Default for XmlWriter {
    fn default() -> Self {
        Self::new()
    }
}

/// 构造一个带文本内容的行内元素.
pub fn inline_text(name: &str, attrs: &[(&str, &str)], text: &str) -> String {
    let mut out = String::new();
    out.push('<');
    out.push_str(name);
    for (key, value) in attrs {
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        out.push_str(&escape_attr(value));
        out.push('"');
    }
    out.push('>');
    out.push_str(&escape_text(text));
    out.push_str("</");
    out.push_str(name);
    out.push('>');
    out
}

/// 构造一个内容已是 XML 的行内元素.
pub fn inline_raw(name: &str, attrs: &[(&str, &str)], inner_xml: &str) -> String {
    let mut out = String::new();
    out.push('<');
    out.push_str(name);
    for (key, value) in attrs {
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        out.push_str(&escape_attr(value));
        out.push('"');
    }
    out.push('>');
    out.push_str(inner_xml);
    out.push_str("</");
    out.push_str(name);
    out.push('>');
    out
}

/// 构造一个空的行内元素.
pub fn inline_empty(name: &str, attrs: &[(&str, &str)]) -> String {
    let mut out = String::new();
    out.push('<');
    out.push_str(name);
    for (key, value) in attrs {
        out.push(' ');
        out.push_str(key);
        out.push_str("=\"");
        out.push_str(&escape_attr(value));
        out.push('"');
    }
    out.push_str("/>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_special_characters() {
        assert_eq!(escape_text("a&b<c>d"), "a&amp;b&lt;c&gt;d");
        assert_eq!(
            escape_attr("a\"b'c<d>e&f"),
            "a&quot;b&apos;c&lt;d&gt;e&amp;f"
        );
        assert_eq!(escape_text("bad\u{0001}char"), "bad&#x1;char");
        assert_eq!(escape_attr("line\nbreak"), "line&#10;break");
    }

    #[test]
    fn keeps_legal_whitespace() {
        assert_eq!(escape_text("a\tb\nc\rd"), "a\tb\nc\rd");
    }

    #[test]
    fn writer_produces_pretty_blocks() {
        let mut w = XmlWriter::new();
        w.declaration();
        w.start("akn:akomaNtoso", &[("xmlns:akn", "urn:akn")]);
        w.start("akn:doc", &[("name", "通知")]);
        w.text_element("akn:p", &[("gbt:role", "title")], "关于 A&B 的通知");
        w.empty("akn:br", &[]);
        w.end("akn:doc");
        w.end("akn:akomaNtoso");
        let xml = w.finish();
        assert!(xml.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("<akn:doc name=\"通知\">"));
        assert!(xml.contains("<akn:p gbt:role=\"title\">关于 A&amp;B 的通知</akn:p>"));
        assert!(xml.contains("<akn:br/>"));
    }

    #[test]
    fn inline_helpers_build_well_formed_fragments() {
        let fragment = inline_raw(
            "akn:p",
            &[("eId", "p1")],
            &inline_text("akn:docTitle", &[], "标题"),
        );
        assert_eq!(
            fragment,
            "<akn:p eId=\"p1\"><akn:docTitle>标题</akn:docTitle></akn:p>"
        );
        assert_eq!(
            inline_empty("akn:p", &[("eId", "p2")]),
            "<akn:p eId=\"p2\"/>"
        );
    }
}

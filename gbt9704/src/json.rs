//! 无第三方依赖的 JSON 解析与序列化.
//!
//! 为什么不用 serde: 本仓库的其它子项目均保持零依赖; 公文输入数据只涉及 JSON 的基础类型,
//! 标准库足够. 本文件只实现 RFC 8259 的完整语法(含 `\uXXXX` 代理对), 不做 schema 校验.
//!
//! 坑:
//! - 字符串中的非 ASCII 字符按 UTF-8 原样保留, 不能按字节切;
//! - 数字统一存为 f64, 年份/序号等整数字段由 `model` 层再做范围校验;
//! - 对象保留插入顺序, 便于 `example` 命令输出稳定的示例文件.

use std::fmt;

/// JSON 值.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(Vec<(String, Value)>),
}

impl Value {
    /// 按 key 取对象成员; 非对象返回 None.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(members) => members.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// 长度: 对象/数组为成员数, 字符串为字符数; 标量返回 None.
    pub fn len(&self) -> Option<usize> {
        match self {
            Value::Object(members) => Some(members.len()),
            Value::Array(items) => Some(items.len()),
            Value::String(s) => Some(s.chars().count()),
            _ => None,
        }
    }

    /// 长度是否为 0; 无法定义长度(null/boolean/number)时返回 false.
    pub fn is_empty(&self) -> bool {
        self.len() == Some(0)
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&[(String, Value)]> {
        match self {
            Value::Object(members) => Some(members),
            _ => None,
        }
    }

    /// 该值的 JSON 类型名, 仅用于报错信息.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "boolean",
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// 构造字符串值的便捷函数.
    pub fn string<S: Into<String>>(s: S) -> Value {
        Value::String(s.into())
    }

    /// 构造对象值的便捷函数.
    pub fn object<S: Into<String>>(members: Vec<(S, Value)>) -> Value {
        Value::Object(members.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
}

/// 解析错误, 带字符偏移, 便于定位手写 JSON 的问题.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub message: String,
    pub offset: usize,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "JSON 解析失败(字符偏移 {}): {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for ParseError {}

/// 解析一个完整 JSON 文档; 末尾只允许空白.
pub fn parse(input: &str) -> Result<Value, ParseError> {
    let chars: Vec<char> = input.chars().collect();
    let mut parser = Parser {
        chars: &chars,
        pos: 0,
    };
    parser.skip_ws();
    let value = parser.parse_value()?;
    parser.skip_ws();
    if parser.pos != parser.chars.len() {
        return Err(parser.error("JSON 文档结束后仍有多余字符"));
    }
    Ok(value)
}

struct Parser<'a> {
    chars: &'a [char],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn error(&self, message: &str) -> ParseError {
        ParseError {
            message: message.to_string(),
            offset: self.pos,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek();
        if c.is_some() {
            self.pos += 1;
        }
        c
    }

    fn skip_ws(&mut self) {
        while let Some(c) = self.peek() {
            if matches!(c, ' ' | '\t' | '\n' | '\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), ParseError> {
        if self.peek() == Some(expected) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(&format!("期望字符 {expected:?}")))
        }
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        self.skip_ws();
        match self.peek() {
            None => Err(self.error("意外的文档结束")),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('"') => self.parse_string().map(Value::String),
            Some('t') => self.parse_literal("true", Value::Bool(true)),
            Some('f') => self.parse_literal("false", Value::Bool(false)),
            Some('n') => self.parse_literal("null", Value::Null),
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(self.error(&format!("非法字符 {c:?}"))),
        }
    }

    fn parse_literal(&mut self, literal: &str, value: Value) -> Result<Value, ParseError> {
        for expected in literal.chars() {
            if self.bump() != Some(expected) {
                return Err(self.error(&format!("期望字面量 {literal:?}")));
            }
        }
        Ok(value)
    }

    fn parse_object(&mut self) -> Result<Value, ParseError> {
        self.expect('{')?;
        let mut members: Vec<(String, Value)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some('}') {
            self.pos += 1;
            return Ok(Value::Object(members));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some('"') {
                return Err(self.error("对象的键必须是字符串"));
            }
            let key = self.parse_string()?;
            // 重复键几乎总是手写输入错误; 直接报错比"后者覆盖前者"更安全.
            if members.iter().any(|(k, _)| *k == key) {
                return Err(self.error(&format!("对象中出现重复键 {key:?}")));
            }
            self.skip_ws();
            self.expect(':')?;
            let value = self.parse_value()?;
            members.push((key, value));
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some('}') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.error("对象中期望 ',' 或 '}'")),
            }
        }
        Ok(Value::Object(members))
    }

    fn parse_array(&mut self) -> Result<Value, ParseError> {
        self.expect('[')?;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(']') {
            self.pos += 1;
            return Ok(Value::Array(items));
        }
        loop {
            let value = self.parse_value()?;
            items.push(value);
            self.skip_ws();
            match self.peek() {
                Some(',') => {
                    self.pos += 1;
                    continue;
                }
                Some(']') => {
                    self.pos += 1;
                    break;
                }
                _ => return Err(self.error("数组中期望 ',' 或 ']'")),
            }
        }
        Ok(Value::Array(items))
    }

    fn parse_string(&mut self) -> Result<String, ParseError> {
        self.expect('"')?;
        let mut out = String::new();
        loop {
            let c = self.bump().ok_or_else(|| self.error("字符串未闭合"))?;
            match c {
                '"' => return Ok(out),
                '\\' => {
                    let escaped = self.bump().ok_or_else(|| self.error("转义序列未完成"))?;
                    match escaped {
                        '"' => out.push('"'),
                        '\\' => out.push('\\'),
                        '/' => out.push('/'),
                        'b' => out.push('\u{0008}'),
                        'f' => out.push('\u{000c}'),
                        'n' => out.push('\n'),
                        'r' => out.push('\r'),
                        't' => out.push('\t'),
                        'u' => {
                            let first = self.parse_hex4()?;
                            let code = if (0xD800..=0xDBFF).contains(&first) {
                                // UTF-16 高代理项必须跟一个低代理项.
                                if self.bump() != Some('\\') || self.bump() != Some('u') {
                                    return Err(self.error("高代理项后必须跟 \\u 低代理项"));
                                }
                                let second = self.parse_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&second) {
                                    return Err(self.error("低代理项范围非法"));
                                }
                                0x10000 + ((first - 0xD800) << 10) + (second - 0xDC00)
                            } else if (0xDC00..=0xDFFF).contains(&first) {
                                return Err(self.error("孤立的低代理项"));
                            } else {
                                first
                            };
                            let ch = char::from_u32(code)
                                .ok_or_else(|| self.error("非法 Unicode 码点"))?;
                            out.push(ch);
                        }
                        other => return Err(self.error(&format!("未知转义 \\{other}"))),
                    }
                }
                // 未转义的控制字符在 JSON 中非法.
                '\u{0000}'..='\u{001f}' => {
                    return Err(self.error("字符串中包含未转义的控制字符"));
                }
                other => out.push(other),
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u32, ParseError> {
        let mut value = 0u32;
        for _ in 0..4 {
            let c = self
                .bump()
                .ok_or_else(|| self.error("\\u 转义缺少 4 位十六进制"))?;
            let digit = c
                .to_digit(16)
                .ok_or_else(|| self.error("\\u 转义含非十六进制字符"))?;
            value = (value << 4) | digit;
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<Value, ParseError> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.pos += 1;
        }
        // 整数部分: 0 或 [1-9][0-9]*
        match self.peek() {
            Some('0') => {
                self.pos += 1;
            }
            Some(c) if c.is_ascii_digit() => {
                while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.error("数字缺少整数部分")),
        }
        // 小数部分
        if self.peek() == Some('.') {
            self.pos += 1;
            if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                return Err(self.error("小数点后必须有数字"));
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        // 指数部分
        if matches!(self.peek(), Some('e' | 'E')) {
            self.pos += 1;
            if matches!(self.peek(), Some('+' | '-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                return Err(self.error("指数部分必须有数字"));
            }
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.pos += 1;
            }
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        let number: f64 = text.parse().map_err(|_| self.error("数字超出可表示范围"))?;
        Ok(Value::Number(number))
    }
}

/// 把值序列化为带 2 空格缩进的 JSON 文本.
pub fn to_pretty(value: &Value) -> String {
    let mut out = String::new();
    write_value(value, 0, &mut out);
    out.push('\n');
    out
}

fn write_value(value: &Value, depth: usize, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => {
            // f64 的 Display 对 1.0 输出 "1", 对 0.1 输出 "0.1"; 与 JSON 语法兼容.
            debug_assert!(n.is_finite(), "JSON 不支持 NaN/Infinity");
            out.push_str(&format!("{n}"));
        }
        Value::String(s) => write_string(s, out),
        Value::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push_str("[\n");
            for (index, item) in items.iter().enumerate() {
                push_indent(depth + 1, out);
                write_value(item, depth + 1, out);
                if index + 1 != items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(depth, out);
            out.push(']');
        }
        Value::Object(members) => {
            if members.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{\n");
            for (index, (key, item)) in members.iter().enumerate() {
                push_indent(depth + 1, out);
                write_string(key, out);
                out.push_str(": ");
                write_value(item, depth + 1, out);
                if index + 1 != members.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            push_indent(depth, out);
            out.push('}');
        }
    }
}

fn push_indent(depth: usize, out: &mut String) {
    for _ in 0..depth {
        out.push_str("  ");
    }
}

fn write_string(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_scalars() {
        assert_eq!(parse("null").unwrap(), Value::Null);
        assert_eq!(parse("true").unwrap(), Value::Bool(true));
        assert_eq!(parse("-12.5e2").unwrap(), Value::Number(-1250.0));
        assert_eq!(parse("\"中文\"").unwrap(), Value::String("中文".into()));
    }

    #[test]
    fn parses_nested_and_preserves_order() {
        let value = parse(r#"{"b": [1, 2, {"c": false}], "a": "x"}"#).unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, ["b", "a"]);
        assert_eq!(value.get("a").unwrap().as_str(), Some("x"));
    }

    #[test]
    fn parses_unicode_escapes_and_surrogates() {
        assert_eq!(
            parse(r#""\u4e2d\u6587""#).unwrap(),
            Value::String("中文".into())
        );
        // U+1F600 (😀) 的代理对
        assert_eq!(
            parse(r#""\ud83d\ude00""#).unwrap(),
            Value::String("😀".into())
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse("").is_err());
        assert!(parse("{").is_err());
        assert!(parse("{\"a\":1,}").is_err());
        assert!(parse("{\"a\":1 \"b\":2}").is_err());
        assert!(parse("[1 2]").is_err());
        assert!(parse("01").is_err());
        assert!(parse("\"a\\q\"").is_err());
    }

    #[test]
    fn rejects_duplicate_keys() {
        assert!(parse(r#"{"a": 1, "a": 2}"#).is_err());
    }

    #[test]
    fn round_trips_via_pretty() {
        let text = r#"{"title":"关于测试的通知","serial":1,"flags":[true,null],"empty":{}}"#;
        let value = parse(text).unwrap();
        let pretty = to_pretty(&value);
        assert_eq!(parse(&pretty).unwrap(), value);
    }

    #[test]
    fn pretty_escapes_controls() {
        let value = Value::String("a\nb\tc".into());
        assert_eq!(to_pretty(&value), "\"a\\nb\\tc\"\n");
    }
}

//! 公文输入模型与 JSON 读写.
//!
//! 这是"生成 AKN"的入口数据: 只表达公文语义 + 少量版式覆盖, 不承载渲染细节.
//! 版式(字体/位置/页码/版记线等)由 `gbt9704` 模块按标准补齐, 最终写入 AKN 的
//! `presentation`/`proprietary` 扩展块.
//!
//! JSON 字段命名使用 snake_case; 文档见 `gbt9704/README.md`.
//!
//! 坑:
//! - `body` 的 `level` 只表示"结构层次序数"层级 0~4, 0 是普通自然段;
//! - 成文日期在 JSON 中用 ISO `YYYY-MM-DD`, 渲染时才转成"2012年7月1日";
//! - 发文字号拆成 agency/year/serial, 便于生成六角括号写法与 FRBR IRI.

use std::fmt;

use crate::gbt9704::FontSlot;
use crate::gbt9704::geometry::{
    self, PAPER_HEIGHT_MM, PAPER_WIDTH_MM, TEXT_HEIGHT_MM, TEXT_WIDTH_MM, char_pitch_mm,
    line_pitch_mm,
};
use crate::json::{self, Value};

/// 公文文种: 《党政机关公文处理工作条例》规定的 15 种.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    JueYi,
    JueDing,
    MingLing,
    GongBao,
    GongGao,
    TongGao,
    YiJian,
    TongZhi,
    TongBao,
    BaoGao,
    QingShi,
    PiFu,
    YiAn,
    Han,
    JiYao,
}

impl DocKind {
    pub const ALL: [DocKind; 15] = [
        DocKind::JueYi,
        DocKind::JueDing,
        DocKind::MingLing,
        DocKind::GongBao,
        DocKind::GongGao,
        DocKind::TongGao,
        DocKind::YiJian,
        DocKind::TongZhi,
        DocKind::TongBao,
        DocKind::BaoGao,
        DocKind::QingShi,
        DocKind::PiFu,
        DocKind::YiAn,
        DocKind::Han,
        DocKind::JiYao,
    ];

    /// 标准文种名称.
    pub fn zh(self) -> &'static str {
        match self {
            DocKind::JueYi => "决议",
            DocKind::JueDing => "决定",
            DocKind::MingLing => "命令(令)",
            DocKind::GongBao => "公报",
            DocKind::GongGao => "公告",
            DocKind::TongGao => "通告",
            DocKind::YiJian => "意见",
            DocKind::TongZhi => "通知",
            DocKind::TongBao => "通报",
            DocKind::BaoGao => "报告",
            DocKind::QingShi => "请示",
            DocKind::PiFu => "批复",
            DocKind::YiAn => "议案",
            DocKind::Han => "函",
            DocKind::JiYao => "纪要",
        }
    }

    /// 用于 FRBR IRI 的 ASCII slug.
    pub fn slug(self) -> &'static str {
        match self {
            DocKind::JueYi => "jueyi",
            DocKind::JueDing => "jueding",
            DocKind::MingLing => "mingling",
            DocKind::GongBao => "gongbao",
            DocKind::GongGao => "gonggao",
            DocKind::TongGao => "tonggao",
            DocKind::YiJian => "yijian",
            DocKind::TongZhi => "tongzhi",
            DocKind::TongBao => "tongbao",
            DocKind::BaoGao => "baogao",
            DocKind::QingShi => "qingshi",
            DocKind::PiFu => "pifu",
            DocKind::YiAn => "yian",
            DocKind::Han => "han",
            DocKind::JiYao => "jiyao",
        }
    }

    pub fn from_zh(name: &str) -> Option<Self> {
        // "命令" 与 "命令(令)" 都接受.
        let normalized = name.trim();
        if normalized == "命令" || normalized == "命令(令)" || normalized == "命令（令）"
        {
            return Some(DocKind::MingLing);
        }
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.zh() == normalized)
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|kind| kind.slug() == slug.trim())
    }

    /// 上行文: 报告、请示. 上行文应标注签发人(7.2.6).
    pub fn is_upward(self) -> bool {
        matches!(self, DocKind::BaoGao | DocKind::QingShi)
    }
}

impl fmt::Display for DocKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.zh())
    }
}

/// 公文格式: 通用格式 + 第 10 章的三种特定格式.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// 通用格式(版心 156×225, 7.2~7.5).
    Standard,
    /// 10.1 信函格式.
    Letter,
    /// 10.2 命令(令)格式.
    Order,
    /// 10.3 纪要格式.
    Minutes,
}

impl Format {
    pub fn id(self) -> &'static str {
        match self {
            Format::Standard => "standard",
            Format::Letter => "letter",
            Format::Order => "order",
            Format::Minutes => "minutes",
        }
    }

    pub fn zh(self) -> &'static str {
        match self {
            Format::Standard => "通用格式",
            Format::Letter => "信函格式",
            Format::Order => "命令(令)格式",
            Format::Minutes => "纪要格式",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim() {
            "standard" => Some(Format::Standard),
            "letter" => Some(Format::Letter),
            "order" => Some(Format::Order),
            "minutes" => Some(Format::Minutes),
            _ => None,
        }
    }
}

impl fmt::Display for Format {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.zh())
    }
}

/// 密级.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecrecyLevel {
    JueMi,
    JiMi,
    MiMi,
}

impl SecrecyLevel {
    pub fn zh(self) -> &'static str {
        match self {
            SecrecyLevel::JueMi => "绝密",
            SecrecyLevel::JiMi => "机密",
            SecrecyLevel::MiMi => "秘密",
        }
    }

    pub fn from_zh(name: &str) -> Option<Self> {
        match name.trim() {
            "绝密" => Some(SecrecyLevel::JueMi),
            "机密" => Some(SecrecyLevel::JiMi),
            "秘密" => Some(SecrecyLevel::MiMi),
            _ => None,
        }
    }
}

/// 密级和保密期限. 标准 7.2.2 用"★"连接, 例如"机密★1年".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Secrecy {
    pub level: SecrecyLevel,
    pub term: Option<String>,
}

impl Secrecy {
    /// 呈现文本: "机密★1年" 或 "机密".
    pub fn display(&self) -> String {
        match &self.term {
            Some(term) if !term.is_empty() => {
                format!("{}★{}", self.level.zh(), term)
            }
            _ => self.level.zh().to_string(),
        }
    }
}

/// 发文机关: 全称/规范化简称, 以及可选的发文机关标志文本.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issuer {
    /// 发文机关名称; 联合行文时按顺序排列, 第一个为主办机关.
    pub names: Vec<String>,
    /// 发文机关标志文本(含"文件"二字); 为空时按 `names` 自动推导.
    pub mark: Option<String>,
}

impl Issuer {
    /// 7.2.4 发文机关标志文本(通用格式).
    ///
    /// 单一机关: "机关名称文件"; 联合行文: 标准要求"文件"二字置于发文机关名称右侧,
    /// 此处返回逻辑文本(名称用空格分隔 + "文件"), 具体分行/堆叠由渲染层处理.
    pub fn mark_text(&self) -> String {
        self.mark_text_for(Format::Standard)
    }

    /// 按公文格式推导发文机关标志(第 10 章三种特定格式的组成规则不同).
    ///
    /// - 通用: 机关名称 + "文件";
    /// - 信函(10.1): 机关全称/规范化简称, 联合行文用主办机关, 不加"文件";
    /// - 命令(令)(10.2): 机关全称 + "命令"或"令";
    /// - 纪要(10.3): "×××××纪要".
    pub fn mark_text_for(&self, format: Format) -> String {
        if let Some(mark) = &self.mark {
            return mark.clone();
        }
        match format {
            Format::Standard => {
                if self.names.len() == 1 {
                    format!("{}文件", self.names[0])
                } else {
                    format!("{}文件", self.names.join(" "))
                }
            }
            Format::Letter => self.names.first().cloned().unwrap_or_default(),
            Format::Order => {
                format!("{}命令", self.names.first().cloned().unwrap_or_default())
            }
            Format::Minutes => {
                format!("{}纪要", self.names.first().cloned().unwrap_or_default())
            }
        }
    }
}

/// 发文字号: 机关代字 + 年份 + 顺序号, 例如 "国办发〔2012〕1号".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocNumber {
    pub agency: String,
    pub year: u16,
    pub serial: u32,
    /// 发文字号的显式呈现文本; 为空时按标准通用写法生成.
    ///
    /// 为什么需要覆盖: 标准只给"机关代字〔年〕序号号"一种写法, 部分机关用
    /// "机关名称〔年〕第001号"(命令式编号), 渲染层需要按输入原文排版.
    pub text: Option<String>,
}

impl DocNumber {
    /// 标准呈现形式(六角括号、年份全称、顺序号不编虚位、加"号"), 可被 `text` 覆盖.
    pub fn display(&self) -> String {
        if let Some(text) = &self.text
            && !text.trim().is_empty()
        {
            return text.clone();
        }
        format!("{}〔{}〕{}号", self.agency, self.year, self.serial)
    }
}

/// 签发人(上行文).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signer {
    pub name: String,
    /// 职务, 例如"部长"; 用于签名章情形(7.3.5.3).
    pub title: Option<String>,
}

/// 正文块: 自然段或结构层次标题.
///
/// - `level == 0`: 普通自然段(3 号仿宋, 首行左空二字);
/// - `level == 1`: "一、" 层(3 号黑体);
/// - `level == 2`: "(一)" 层(3 号楷体);
/// - `level == 3`: "1." 层(3 号仿宋);
/// - `level == 4`: "(1)" 层(3 号仿宋).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyBlock {
    pub level: u8,
    pub num: Option<String>,
    pub text: String,
}

impl BodyBlock {
    pub fn paragraph(text: impl Into<String>) -> Self {
        Self {
            level: 0,
            num: None,
            text: text.into(),
        }
    }

    pub fn heading(level: u8, num: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            level,
            num: Some(num.into()),
            text: text.into(),
        }
    }

    /// 该块的字体槽位(7.3.3: 第一层黑体, 第二层楷体, 第三/四层仿宋).
    pub fn font_slot(&self) -> FontSlot {
        match self.level {
            0 | 3 | 4 => FontSlot::Body,
            1 => FontSlot::Heiti,
            2 => FontSlot::Kaiti,
            _ => FontSlot::Body,
        }
    }

    pub fn is_heading(&self) -> bool {
        self.level > 0
    }
}

/// 附件说明项(7.3.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttachmentNote {
    /// 附件顺序号; 单个附件时可省略.
    pub serial: Option<u32>,
    pub name: String,
}

impl AttachmentNote {
    pub fn display(&self) -> String {
        match self.serial {
            Some(serial) => format!("{serial}.{}", self.name),
            None => self.name.clone(),
        }
    }
}

/// 印章/署名情形(7.3.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SealMode {
    /// 7.3.5.1 加盖印章.
    WithSeal,
    /// 7.3.5.2 不加盖印章.
    WithoutSeal,
    /// 7.3.5.3 加盖签发人签名章.
    SignatureSeal,
}

impl SealMode {
    pub fn id(self) -> &'static str {
        match self {
            SealMode::WithSeal => "with_seal",
            SealMode::WithoutSeal => "without_seal",
            SealMode::SignatureSeal => "signature_seal",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim() {
            "with_seal" => Some(SealMode::WithSeal),
            "without_seal" => Some(SealMode::WithoutSeal),
            "signature_seal" => Some(SealMode::SignatureSeal),
            _ => None,
        }
    }

    pub fn zh(self) -> &'static str {
        match self {
            SealMode::WithSeal => "加盖印章",
            SealMode::WithoutSeal => "不加盖印章",
            SealMode::SignatureSeal => "加盖签发人签名章",
        }
    }
}

/// 一个印章/签名章实例. 印章是图形资源, 标准只规定颜色(红色)与相对位置.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Seal {
    /// 印章图片路径(阶段 2 使用); 无图片时渲染层留出印章位置.
    pub image: Option<String>,
    /// 对应发文机关署名(联合行文时用于一一对应).
    pub issuer: Option<String>,
    /// 对应签发人姓名(签名章情形).
    pub signer: Option<String>,
    /// 签发人职务(签名章情形, 7.3.5.3).
    pub title: Option<String>,
    /// 印章编号(圆形公章下方的阿拉伯数字); 为空时渲染层不画编号.
    pub number: Option<String>,
}

/// 发文机关署名、成文日期和印章(7.3.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// ISO 日期 `YYYY-MM-DD`; 渲染时转为"2012年7月1日".
    pub date: String,
    /// 发文机关署名; 为空时回退到 `Issuer::names`.
    pub issuer_names: Vec<String>,
    pub mode: SealMode,
    pub seals: Vec<Seal>,
    /// 成文日期的显式呈现文本; 为空时由 `date` 转成"2026年11月9日".
    ///
    /// 为什么需要覆盖: 式样图与不少机关习惯用汉字数字("二〇二六年十一月九日"),
    /// 标准 7.3.5.4 要求阿拉伯数字, 渲染层按输入原文排.
    pub date_text: Option<String>,
}

/// 附件(7.3.7): 另面编排, 在版记之前.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attachment {
    pub serial: u32,
    pub title: String,
    /// 附件正文段落(附件格式要求同正文; 本模型不做层次结构, 保持简单).
    pub paragraphs: Vec<String>,
    /// 附件与正文不能一起装订时, 需在附件左上角标注发文字号+"附件"及顺序号(7.3.7).
    pub standalone: bool,
}

/// 纪要的出席/请假/列席人员名单(10.3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Attendance {
    pub present: Vec<String>,
    pub absent: Vec<String>,
    pub observers: Vec<String>,
}

/// 印发机关和印发日期(7.4.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Printing {
    pub issuer: String,
    pub date: String,
}

/// 页码配置(7.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageConfig {
    pub enabled: bool,
}

impl Default for PageConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

/// 版心几何覆盖. 默认值严格等于 GB/T 9704-2012 的 5.2.1/5.2.3;
/// 非默认值会在校验时给出警告(标准允许±1 mm 误差与"特定情况适当调整").
#[derive(Debug, Clone, PartialEq)]
pub struct LayoutSpec {
    pub margin_top_mm: f64,
    pub margin_left_mm: f64,
    pub text_width_mm: f64,
    pub text_height_mm: f64,
    pub line_pitch_mm: f64,
    pub char_pitch_mm: f64,
}

impl Default for LayoutSpec {
    fn default() -> Self {
        Self {
            margin_top_mm: geometry::MARGIN_TOP_MM,
            margin_left_mm: geometry::MARGIN_LEFT_MM,
            text_width_mm: TEXT_WIDTH_MM,
            text_height_mm: TEXT_HEIGHT_MM,
            line_pitch_mm: line_pitch_mm(),
            char_pitch_mm: char_pitch_mm(),
        }
    }
}

impl LayoutSpec {
    /// 是否严格符合标准 5.2.1/5.2.3 (容差 1e-6 mm).
    pub fn is_standard(&self) -> bool {
        (self.margin_top_mm - geometry::MARGIN_TOP_MM).abs() < 1e-6
            && (self.margin_left_mm - geometry::MARGIN_LEFT_MM).abs() < 1e-6
            && (self.text_width_mm - TEXT_WIDTH_MM).abs() < 1e-6
            && (self.text_height_mm - TEXT_HEIGHT_MM).abs() < 1e-6
            && (self.line_pitch_mm - line_pitch_mm()).abs() < 1e-6
            && (self.char_pitch_mm - char_pitch_mm()).abs() < 1e-6
    }

    /// 纸张尺寸(mm).
    pub fn paper_size_mm(&self) -> (f64, f64) {
        (PAPER_WIDTH_MM, PAPER_HEIGHT_MM)
    }

    /// 右白边/下白边(mm).
    pub fn margin_right_mm(&self) -> f64 {
        PAPER_WIDTH_MM - self.margin_left_mm - self.text_width_mm
    }

    pub fn margin_bottom_mm(&self) -> f64 {
        PAPER_HEIGHT_MM - self.margin_top_mm - self.text_height_mm
    }
}

/// 一份完整的公文.
#[derive(Debug, Clone, PartialEq)]
pub struct GbDocument {
    pub doc_kind: DocKind,
    pub format: Format,
    /// 可选的 FRBR 基础 IRI; 为空时按 `/{country}/gbt9704/{slug}/{year}/{serial}` 生成.
    pub frbr_base: Option<String>,
    pub copy_number: Option<String>,
    pub secrecy: Option<Secrecy>,
    pub urgency: Option<String>,
    pub issuer: Issuer,
    pub doc_number: Option<DocNumber>,
    /// 命令(令)的令号(10.2); 为空且 format=order 时由发文字号顺序号推导.
    pub order_number: Option<String>,
    pub signers: Vec<Signer>,
    pub title: String,
    pub addressees: Vec<String>,
    /// 主送机关是否移至版记(7.4.2, 仅在首页放不下正文时使用).
    pub addressees_in_record: bool,
    pub body: Vec<BodyBlock>,
    pub attachment_notes: Vec<AttachmentNote>,
    pub signature: Signature,
    pub note: Option<String>,
    pub attachments: Vec<Attachment>,
    pub cc: Vec<String>,
    pub printing: Option<Printing>,
    pub attendance: Option<Attendance>,
    pub page: PageConfig,
    pub layout: LayoutSpec,
}

/// 诊断级别.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn zh(self) -> &'static str {
        match self {
            Severity::Error => "错误",
            Severity::Warning => "警告",
        }
    }
}

/// 一条校验诊断.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub field: String,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.severity.zh(),
            self.field,
            self.message
        )
    }
}

impl GbDocument {
    /// 是否含阻断生成的错误.
    pub fn has_errors(diagnostics: &[Diagnostic]) -> bool {
        diagnostics.iter().any(|d| d.severity == Severity::Error)
    }

    /// 结构 + 标准符合性校验.
    pub fn validate(&self) -> Vec<Diagnostic> {
        let mut out = Vec::new();
        let mut push = |severity, field: &str, message: String| {
            out.push(Diagnostic {
                severity,
                field: field.to_string(),
                message,
            });
        };

        if self.title.trim().is_empty() {
            push(Severity::Error, "title", "标题不能为空".into());
        }
        if self.issuer.names.is_empty() {
            push(
                Severity::Error,
                "issuer.names",
                "发文机关名称不能为空".into(),
            );
        }
        if !is_iso_date(&self.signature.date) {
            push(
                Severity::Error,
                "signature.date",
                format!("成文日期必须是合法的 YYYY-MM-DD: {}", self.signature.date),
            );
        }
        if let Some(printing) = &self.printing
            && !is_iso_date(&printing.date)
        {
            push(
                Severity::Error,
                "printing.date",
                format!("印发日期必须是合法的 YYYY-MM-DD: {}", printing.date),
            );
        }
        if self.body.is_empty() {
            push(
                Severity::Warning,
                "body",
                "正文为空; 7.3.3 要求首页必须显示正文".into(),
            );
        }
        for (index, block) in self.body.iter().enumerate() {
            if block.level > 4 {
                push(
                    Severity::Error,
                    &format!("body[{index}].level"),
                    format!("结构层次只允许 0~4 层, 实际为 {}", block.level),
                );
            }
            if block.level > 0 && block.num.as_deref().unwrap_or("").trim().is_empty() {
                push(
                    Severity::Error,
                    &format!("body[{index}].num"),
                    "结构层次标题必须给出序数(num)".into(),
                );
            }
            if block.level == 0 && block.num.is_some() {
                push(
                    Severity::Warning,
                    &format!("body[{index}].num"),
                    "普通自然段(level=0)不应带序数; 若为层次标题请设置 level".into(),
                );
            }
            if block.text.trim().is_empty() {
                push(
                    Severity::Warning,
                    &format!("body[{index}].text"),
                    "段落文本为空".into(),
                );
            }
        }
        if let Some(number) = &self.doc_number {
            if number.agency.trim().is_empty() {
                push(
                    Severity::Error,
                    "doc_number.agency",
                    "机关代字不能为空".into(),
                );
            }
            if !(1900..=9999).contains(&number.year) {
                push(
                    Severity::Error,
                    "doc_number.year",
                    format!("发文年份不合理: {}", number.year),
                );
            }
            if number.serial == 0 {
                push(
                    Severity::Error,
                    "doc_number.serial",
                    "发文顺序号不能为 0".into(),
                );
            }
        } else if self.format != Format::Minutes {
            push(
                Severity::Warning,
                "doc_number",
                "未提供发文字号; 7.2.5 规定版头应编排发文字号".into(),
            );
        }
        if self.format == Format::Order && self.order_number.is_none() && self.doc_number.is_none()
        {
            push(
                Severity::Warning,
                "order_number",
                "命令(令)格式应在发文机关标志下空二行编排令号(10.2)".into(),
            );
        }
        if self.addressees_in_record && self.addressees.is_empty() {
            push(
                Severity::Warning,
                "addressees_in_record",
                "addressees_in_record=true 但没有可移入版记的主送机关".into(),
            );
        }
        if let Some(copy_number) = &self.copy_number
            && (copy_number.len() != 6 || !copy_number.chars().all(|c| c.is_ascii_digit()))
        {
            push(
                Severity::Warning,
                "copy_number",
                format!("7.2.1 建议用 6 位阿拉伯数字, 当前为 {copy_number:?}"),
            );
        }
        if let Some(note) = &self.note
            && (!note.starts_with('（') || !note.ends_with('）'))
        {
            push(Severity::Warning, "note", "7.3.6 要求附注加圆括号".into());
        }
        // 特定格式与文种的兼容性.
        match self.format {
            Format::Order if self.doc_kind != DocKind::MingLing => push(
                Severity::Warning,
                "format",
                "命令(令)格式通常只用于命令(令)".into(),
            ),
            Format::Minutes if self.doc_kind != DocKind::JiYao => {
                push(Severity::Warning, "format", "纪要格式通常只用于纪要".into())
            }
            Format::Letter if self.doc_kind != DocKind::Han => push(
                Severity::Warning,
                "format",
                "信函格式主要用于函(10.1); 其他文种使用需确认".into(),
            ),
            _ => {}
        }
        if self.doc_kind.is_upward() && self.signers.is_empty() {
            push(
                Severity::Warning,
                "signers",
                "上行文(报告/请示)应标注签发人(7.2.6)".into(),
            );
        }
        if self.format == Format::Order && self.signers.is_empty() {
            push(
                Severity::Warning,
                "signers",
                "命令(令)需编排签发人职务、签名章和成文日期(10.2→7.3.5.3)".into(),
            );
        }
        if self.signature.mode == SealMode::WithSeal && self.signature.seals.is_empty() {
            push(
                Severity::Warning,
                "signature.seals",
                "加盖印章情形未提供印章图片(阶段 2 将留白)".into(),
            );
        }
        if self.signature.mode == SealMode::SignatureSeal && self.signers.is_empty() {
            push(
                Severity::Warning,
                "signers",
                "加盖签名章情形未提供签发人".into(),
            );
        }
        if !self.cc.is_empty() && self.printing.is_none() {
            push(
                Severity::Warning,
                "printing",
                "版记有抄送机关时, 通常也应编排印发机关和印发日期(7.4.3)".into(),
            );
        }
        if self.format == Format::Letter {
            if self.page.enabled {
                push(
                    Severity::Warning,
                    "page.enabled",
                    "信函格式首页不显示页码(10.1); 生成时按信函规则处理".into(),
                );
            }
            if !self.cc.is_empty() {
                push(
                    Severity::Warning,
                    "cc",
                    "信函格式版记不加印发机关和印发日期、分隔线(10.1)".into(),
                );
            }
        }
        if !self.layout.is_standard() {
            push(
                Severity::Warning,
                "layout",
                "版心/网格偏离标准 5.2.1/5.2.3; 标准允许±1 mm 与特定情况适当调整".into(),
            );
        }
        // 附件说明与附件实体的一致性.
        for note in &self.attachment_notes {
            if let Some(serial) = note.serial {
                let found = self
                    .attachments
                    .iter()
                    .any(|a| a.serial == serial && a.title == note.name);
                if !found {
                    push(
                        Severity::Warning,
                        "attachment_notes",
                        format!("附件说明第 {serial} 项没有对应附件(附件顺序号与标题应一致)"),
                    );
                }
            }
        }
        let mut seen_serial = std::collections::HashSet::new();
        for attachment in &self.attachments {
            if !seen_serial.insert(attachment.serial) {
                push(
                    Severity::Error,
                    "attachments",
                    format!("附件顺序号重复: {}", attachment.serial),
                );
            }
        }
        if self.attendance.is_some() && self.doc_kind != DocKind::JiYao {
            push(
                Severity::Warning,
                "attendance",
                "出席/请假/列席人员名单只用于纪要(10.3)".into(),
            );
        }
        out
    }

    /// 生成/获取 FRBR 基础 IRI.
    pub fn frbr_base_iri(&self) -> String {
        if let Some(base) = &self.frbr_base {
            return base.clone();
        }
        let year = self
            .doc_number
            .as_ref()
            .map(|n| n.year.to_string())
            .unwrap_or_else(|| "0000".to_string());
        let serial = self
            .doc_number
            .as_ref()
            .map(|n| n.serial.to_string())
            .unwrap_or_else(|| "1".to_string());
        format!("/cn/gbt9704/{}/{}/{}", self.doc_kind.slug(), year, serial)
    }

    /// 正文首段是否包含"签发人"标记外的上行文特征(仅用于提示).
    pub fn is_upward(&self) -> bool {
        self.doc_kind.is_upward()
    }

    /// 发文机关署名文本(为空时回退到发文机关名称).
    pub fn signature_issuer_names(&self) -> Vec<String> {
        if self.signature.issuer_names.is_empty() {
            self.issuer.names.clone()
        } else {
            self.signature.issuer_names.clone()
        }
    }

    /// 成文日期的中文呈现(7.3.5.4: 阿拉伯数字, 年月日标全, 月日不编虚位).
    pub fn date_zh(&self) -> String {
        iso_date_to_zh(&self.signature.date).unwrap_or_else(|| self.signature.date.clone())
    }

    /// 从 JSON 值解析公文.
    pub fn from_json(value: &Value) -> Result<Self, String> {
        let object = expect_object(value, "$")?;
        let doc_kind_text = required_str(object, "doc_kind", "$")?;
        let doc_kind = DocKind::from_zh(doc_kind_text)
            .or_else(|| DocKind::from_slug(doc_kind_text))
            .ok_or_else(|| {
                format!(
                    "doc_kind 未知: {doc_kind_text:?}; 允许值: {}",
                    DocKind::ALL
                        .iter()
                        .map(|k| k.zh())
                        .collect::<Vec<_>>()
                        .join("/")
                )
            })?;
        let format = match optional_str(object, "format", "$")? {
            Some(id) => Format::from_id(id).ok_or_else(|| {
                format!("format 未知: {id:?}; 允许 standard/letter/order/minutes")
            })?,
            None => Format::Standard,
        };
        let frbr_base = optional_str(object, "frbr_base", "$")?.map(str::to_string);
        let copy_number = optional_str(object, "copy_number", "$")?.map(str::to_string);
        let secrecy = match optional_value(object, "secrecy") {
            Some(value) => {
                let secrecy_object = expect_object(value, "secrecy")?;
                let level_text = required_str(secrecy_object, "level", "secrecy")?;
                let level = SecrecyLevel::from_zh(level_text).ok_or_else(|| {
                    format!("secrecy.level 未知: {level_text:?}; 允许 绝密/机密/秘密")
                })?;
                let term = optional_str(secrecy_object, "term", "secrecy")?.map(str::to_string);
                Some(Secrecy { level, term })
            }
            None => None,
        };
        let urgency = optional_str(object, "urgency", "$")?.map(str::to_string);
        let issuer = parse_issuer(optional_value(object, "issuer").ok_or("缺少必填字段 issuer")?)?;
        let doc_number = match optional_value(object, "doc_number") {
            Some(value) => Some(parse_doc_number(value)?),
            None => None,
        };
        let order_number = optional_str(object, "order_number", "$")?.map(str::to_string);
        let signers = match optional_value(object, "signers") {
            Some(value) => parse_signers(value)?,
            None => Vec::new(),
        };
        let title = required_str(object, "title", "$")?.to_string();
        let addressees = match optional_value(object, "addressees") {
            Some(value) => parse_string_array(value, "addressees")?,
            None => Vec::new(),
        };
        let addressees_in_record =
            optional_bool(object, "addressees_in_record", "$")?.unwrap_or(false);
        let body = match optional_value(object, "body") {
            Some(value) => parse_body(value)?,
            None => Vec::new(),
        };
        let attachment_notes = match optional_value(object, "attachment_notes") {
            Some(value) => parse_attachment_notes(value)?,
            None => Vec::new(),
        };
        let signature =
            parse_signature(optional_value(object, "signature").ok_or("缺少必填字段 signature")?)?;
        let note = optional_str(object, "note", "$")?.map(str::to_string);
        let attachments = match optional_value(object, "attachments") {
            Some(value) => parse_attachments(value)?,
            None => Vec::new(),
        };
        let cc = match optional_value(object, "cc") {
            Some(value) => parse_string_array(value, "cc")?,
            None => Vec::new(),
        };
        let printing = match optional_value(object, "printing") {
            Some(value) => Some(parse_printing(value)?),
            None => None,
        };
        let attendance = match optional_value(object, "attendance") {
            Some(value) => Some(parse_attendance(value)?),
            None => None,
        };
        let page = match optional_value(object, "page") {
            Some(value) => {
                let page_object = expect_object(value, "page")?;
                PageConfig {
                    enabled: optional_bool(page_object, "enabled", "page")?.unwrap_or(true),
                }
            }
            None => PageConfig::default(),
        };
        let layout = match optional_value(object, "layout") {
            Some(value) => parse_layout(value)?,
            None => LayoutSpec::default(),
        };

        Ok(Self {
            doc_kind,
            format,
            frbr_base,
            copy_number,
            secrecy,
            urgency,
            issuer,
            doc_number,
            order_number,
            signers,
            title,
            addressees,
            addressees_in_record,
            body,
            attachment_notes,
            signature,
            note,
            attachments,
            cc,
            printing,
            attendance,
            page,
            layout,
        })
    }
}

// ---------------------------------------------------------------------------
// JSON 解析辅助
// ---------------------------------------------------------------------------

fn expect_object<'a>(value: &'a Value, path: &str) -> Result<&'a [(String, Value)], String> {
    value
        .as_object()
        .ok_or_else(|| format!("{path} 必须是对象, 实际为 {}", value.type_name()))
}

fn optional_value<'a>(object: &'a [(String, Value)], key: &str) -> Option<&'a Value> {
    object.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn required_str<'a>(
    object: &'a [(String, Value)],
    key: &str,
    path: &str,
) -> Result<&'a str, String> {
    match optional_value(object, key) {
        Some(value) => value
            .as_str()
            .ok_or_else(|| format!("{path}.{key} 必须是字符串, 实际为 {}", value.type_name())),
        None => Err(format!("{path}.{key} 缺失")),
    }
}

fn optional_str<'a>(
    object: &'a [(String, Value)],
    key: &str,
    path: &str,
) -> Result<Option<&'a str>, String> {
    match optional_value(object, key) {
        Some(Value::Null) | None => Ok(None),
        Some(value) => value
            .as_str()
            .map(Some)
            .ok_or_else(|| format!("{path}.{key} 必须是字符串, 实际为 {}", value.type_name())),
    }
}

fn optional_bool(
    object: &[(String, Value)],
    key: &str,
    path: &str,
) -> Result<Option<bool>, String> {
    match optional_value(object, key) {
        Some(Value::Null) | None => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| format!("{path}.{key} 必须是布尔值, 实际为 {}", value.type_name())),
    }
}

fn optional_f64(object: &[(String, Value)], key: &str, path: &str) -> Result<Option<f64>, String> {
    match optional_value(object, key) {
        Some(Value::Null) | None => Ok(None),
        Some(value) => value
            .as_f64()
            .map(Some)
            .ok_or_else(|| format!("{path}.{key} 必须是数字, 实际为 {}", value.type_name())),
    }
}

fn required_u32(object: &[(String, Value)], key: &str, path: &str) -> Result<u32, String> {
    let value = optional_value(object, key).ok_or_else(|| format!("{path}.{key} 缺失"))?;
    let number = value
        .as_f64()
        .ok_or_else(|| format!("{path}.{key} 必须是数字, 实际为 {}", value.type_name()))?;
    if number < 0.0 || number.fract() != 0.0 || number > u32::MAX as f64 {
        return Err(format!("{path}.{key} 必须是非负整数, 实际为 {number}"));
    }
    Ok(number as u32)
}

fn parse_string_array(value: &Value, path: &str) -> Result<Vec<String>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("{path} 必须是数组, 实际为 {}", value.type_name()))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_str()
                .map(str::to_string)
                .ok_or_else(|| format!("{path}[{index}] 必须是字符串, 实际为 {}", item.type_name()))
        })
        .collect()
}

fn parse_issuer(value: &Value) -> Result<Issuer, String> {
    let object = expect_object(value, "issuer")?;
    let names = parse_string_array(
        optional_value(object, "names").ok_or("issuer.names 缺失")?,
        "issuer.names",
    )?;
    if names.is_empty() {
        return Err("issuer.names 不能为空".into());
    }
    let mark = optional_str(object, "mark", "issuer")?.map(str::to_string);
    Ok(Issuer { names, mark })
}

fn parse_doc_number(value: &Value) -> Result<DocNumber, String> {
    let object = expect_object(value, "doc_number")?;
    let agency = required_str(object, "agency", "doc_number")?.to_string();
    let year = required_u32(object, "year", "doc_number")?;
    if year > u16::MAX as u32 {
        return Err(format!("doc_number.year 超出范围: {year}"));
    }
    let serial = required_u32(object, "serial", "doc_number")?;
    let text = optional_str(object, "text", "doc_number")?.map(str::to_string);
    Ok(DocNumber {
        agency,
        year: year as u16,
        serial,
        text,
    })
}

fn parse_signers(value: &Value) -> Result<Vec<Signer>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("signers 必须是数组, 实际为 {}", value.type_name()))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let path = format!("signers[{index}]");
            let object = expect_object(item, &path)?;
            let name = required_str(object, "name", &path)?.to_string();
            let title = optional_str(object, "title", &path)?.map(str::to_string);
            Ok(Signer { name, title })
        })
        .collect()
}

fn parse_body(value: &Value) -> Result<Vec<BodyBlock>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("body 必须是数组, 实际为 {}", value.type_name()))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let path = format!("body[{index}]");
            let object = expect_object(item, &path)?;
            let level = match optional_value(object, "level") {
                None => 0u8,
                Some(value) => {
                    let number = value.as_f64().ok_or_else(|| {
                        format!("{path}.level 必须是数字, 实际为 {}", value.type_name())
                    })?;
                    if number < 0.0 || number.fract() != 0.0 || number > 4.0 {
                        return Err(format!("{path}.level 必须是 0~4 的整数"));
                    }
                    number as u8
                }
            };
            let num = optional_str(object, "num", &path)?.map(str::to_string);
            let text = required_str(object, "text", &path)?.to_string();
            Ok(BodyBlock { level, num, text })
        })
        .collect()
}

fn parse_attachment_notes(value: &Value) -> Result<Vec<AttachmentNote>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("attachment_notes 必须是数组, 实际为 {}", value.type_name()))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let path = format!("attachment_notes[{index}]");
            let object = expect_object(item, &path)?;
            let name = required_str(object, "name", &path)?.to_string();
            let serial = match optional_value(object, "serial") {
                Some(Value::Null) | None => None,
                Some(_) => Some(required_u32(object, "serial", &path)?),
            };
            Ok(AttachmentNote { serial, name })
        })
        .collect()
}

fn parse_signature(value: &Value) -> Result<Signature, String> {
    let object = expect_object(value, "signature")?;
    let date = required_str(object, "date", "signature")?.to_string();
    let issuer_names = match optional_value(object, "issuer_names") {
        Some(value) => parse_string_array(value, "signature.issuer_names")?,
        None => Vec::new(),
    };
    let mode = match optional_str(object, "mode", "signature")? {
        Some(id) => SealMode::from_id(id).ok_or_else(|| {
            format!("signature.mode 未知: {id:?}; 允许 with_seal/without_seal/signature_seal")
        })?,
        None => SealMode::WithSeal,
    };
    let seals = match optional_value(object, "seals") {
        Some(value) => {
            let items = value.as_array().ok_or_else(|| {
                format!("signature.seals 必须是数组, 实际为 {}", value.type_name())
            })?;
            items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    let path = format!("signature.seals[{index}]");
                    let object = expect_object(item, &path)?;
                    Ok(Seal {
                        image: optional_str(object, "image", &path)?.map(str::to_string),
                        issuer: optional_str(object, "issuer", &path)?.map(str::to_string),
                        signer: optional_str(object, "signer", &path)?.map(str::to_string),
                        title: optional_str(object, "title", &path)?.map(str::to_string),
                        number: optional_str(object, "number", &path)?.map(str::to_string),
                    })
                })
                .collect::<Result<Vec<_>, String>>()?
        }
        None => Vec::new(),
    };
    let date_text = optional_str(object, "date_text", "signature")?.map(str::to_string);
    Ok(Signature {
        date,
        issuer_names,
        mode,
        seals,
        date_text,
    })
}

fn parse_attachments(value: &Value) -> Result<Vec<Attachment>, String> {
    let items = value
        .as_array()
        .ok_or_else(|| format!("attachments 必须是数组, 实际为 {}", value.type_name()))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let path = format!("attachments[{index}]");
            let object = expect_object(item, &path)?;
            let serial = required_u32(object, "serial", &path)?;
            let title = required_str(object, "title", &path)?.to_string();
            let paragraphs = match optional_value(object, "paragraphs") {
                Some(value) => parse_string_array(value, &format!("{path}.paragraphs"))?,
                None => Vec::new(),
            };
            let standalone = optional_bool(object, "standalone", &path)?.unwrap_or(false);
            Ok(Attachment {
                serial,
                title,
                paragraphs,
                standalone,
            })
        })
        .collect()
}

fn parse_printing(value: &Value) -> Result<Printing, String> {
    let object = expect_object(value, "printing")?;
    let issuer = required_str(object, "issuer", "printing")?.to_string();
    let date = required_str(object, "date", "printing")?.to_string();
    Ok(Printing { issuer, date })
}

fn parse_attendance(value: &Value) -> Result<Attendance, String> {
    let object = expect_object(value, "attendance")?;
    let parse_list = |key: &str| -> Result<Vec<String>, String> {
        match optional_value(object, key) {
            Some(value) => parse_string_array(value, &format!("attendance.{key}")),
            None => Ok(Vec::new()),
        }
    };
    Ok(Attendance {
        present: parse_list("present")?,
        absent: parse_list("absent")?,
        observers: parse_list("observers")?,
    })
}

fn parse_layout(value: &Value) -> Result<LayoutSpec, String> {
    let object = expect_object(value, "layout")?;
    let mut layout = LayoutSpec::default();
    if let Some(mm) = optional_f64(object, "margin_top_mm", "layout")? {
        layout.margin_top_mm = mm;
    }
    if let Some(mm) = optional_f64(object, "margin_left_mm", "layout")? {
        layout.margin_left_mm = mm;
    }
    if let Some(mm) = optional_f64(object, "text_width_mm", "layout")? {
        layout.text_width_mm = mm;
    }
    if let Some(mm) = optional_f64(object, "text_height_mm", "layout")? {
        layout.text_height_mm = mm;
    }
    if let Some(mm) = optional_f64(object, "line_pitch_mm", "layout")? {
        layout.line_pitch_mm = mm;
    }
    if let Some(mm) = optional_f64(object, "char_pitch_mm", "layout")? {
        layout.char_pitch_mm = mm;
    }
    for (key, value) in [
        ("margin_top_mm", layout.margin_top_mm),
        ("margin_left_mm", layout.margin_left_mm),
        ("text_width_mm", layout.text_width_mm),
        ("text_height_mm", layout.text_height_mm),
        ("line_pitch_mm", layout.line_pitch_mm),
        ("char_pitch_mm", layout.char_pitch_mm),
    ] {
        if !value.is_finite() || value <= 0.0 {
            return Err(format!("layout.{key} 必须是正数, 实际为 {value}"));
        }
    }
    Ok(layout)
}

// ---------------------------------------------------------------------------
// 日期
// ---------------------------------------------------------------------------

/// 判断并解析 ISO `YYYY-MM-DD`(含基本范围与月份天数校验, 含闰年).
pub fn parse_iso_date(text: &str) -> Option<(u32, u32, u32)> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let year: u32 = text[0..4].parse().ok()?;
    let month: u32 = text[5..7].parse().ok()?;
    let day: u32 = text[8..10].parse().ok()?;
    if !(1..=12).contains(&month) {
        return None;
    }
    let leap = (year.is_multiple_of(4) && !year.is_multiple_of(100)) || year.is_multiple_of(400);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if !(1..=max_day).contains(&day) {
        return None;
    }
    Some((year, month, day))
}

pub fn is_iso_date(text: &str) -> bool {
    parse_iso_date(text).is_some()
}

/// ISO 日期转中文呈现: "2012-07-01" -> "2012年7月1日" (月日不编虚位, 7.3.5.4).
pub fn iso_date_to_zh(text: &str) -> Option<String> {
    let (year, month, day) = parse_iso_date(text)?;
    Some(format!("{year}年{month}月{day}日"))
}

// ---------------------------------------------------------------------------
// 示例输入
// ---------------------------------------------------------------------------

/// 一份"通知"的完整示例 JSON, 供 `gbt9704 example` 输出与测试使用.
pub const EXAMPLE_JSON: &str = r#"{
  "doc_kind": "通知",
  "format": "standard",
  "copy_number": "000001",
  "secrecy": { "level": "机密", "term": "1年" },
  "urgency": "特急",
  "issuer": { "names": ["国务院办公厅"], "mark": "国务院办公厅文件" },
  "doc_number": { "agency": "国办发", "year": 2012, "serial": 1 },
  "signers": [],
  "title": "国务院办公厅关于进一步加强公文格式规范工作的通知",
  "addressees": ["各省、自治区、直辖市人民政府，国务院各部委、各直属机构"],
  "body": [
    { "text": "为进一步加强公文格式规范工作，经国务院同意，现就有关事项通知如下：" },
    { "level": 1, "num": "一、", "text": "充分认识公文格式规范的重要意义" },
    { "text": "公文格式是公文处理工作的重要组成部分，直接关系公文质量和机关形象。" },
    { "level": 2, "num": "（一）", "text": "统一思想认识" },
    { "text": "各地区、各部门要高度重视公文格式规范化工作。" },
    { "level": 3, "num": "1.", "text": "加强组织领导" },
    { "text": "明确责任分工，抓好工作落实。" },
    { "level": 4, "num": "（1）", "text": "完善工作机制" },
    { "text": "建立健全公文格式审核机制。" },
    { "level": 1, "num": "二、", "text": "严格执行公文格式标准" },
    { "text": "严格按照 GB/T 9704-2012 执行。" }
  ],
  "attachment_notes": [{ "serial": 1, "name": "公文格式自查表" }],
  "signature": {
    "date": "2012-07-01",
    "mode": "with_seal",
    "issuer_names": [],
    "seals": [{ "image": "assets/seal.png", "issuer": "国务院办公厅" }]
  },
  "note": "（此件公开发布）",
  "attachments": [
    { "serial": 1, "title": "公文格式自查表", "paragraphs": ["一、版心尺寸是否符合要求。", "二、字体字号是否符合要求。"] }
  ],
  "cc": ["中共中央办公厅"],
  "printing": { "issuer": "国务院办公厅秘书局", "date": "2012-07-02" },
  "page": { "enabled": true }
}
"#;

/// 取示例文档.
pub fn example_document() -> GbDocument {
    let value = json::parse(EXAMPLE_JSON).expect("内置示例 JSON 必须合法");
    GbDocument::from_json(&value).expect("内置示例必须通过模型解析")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn example_parses_and_validates_without_errors() {
        let document = example_document();
        let diagnostics = document.validate();
        assert!(
            !GbDocument::has_errors(&diagnostics),
            "示例不应有错误: {diagnostics:?}"
        );
        assert_eq!(document.doc_kind, DocKind::TongZhi);
        assert_eq!(
            document.doc_number.as_ref().unwrap().display(),
            "国办发〔2012〕1号"
        );
        assert_eq!(document.date_zh(), "2012年7月1日");
        assert_eq!(document.issuer.mark_text(), "国务院办公厅文件");
    }

    #[test]
    fn date_parsing_handles_leap_years() {
        assert!(is_iso_date("2012-07-01"));
        assert!(is_iso_date("2024-02-29"));
        assert!(!is_iso_date("2023-02-29"));
        assert!(!is_iso_date("2012-13-01"));
        assert!(!is_iso_date("2012-7-1"));
        assert_eq!(
            iso_date_to_zh("2012-07-01").as_deref(),
            Some("2012年7月1日")
        );
    }

    #[test]
    fn body_block_font_slots_follow_7_3_3() {
        assert_eq!(
            BodyBlock::heading(1, "一、", "x").font_slot(),
            FontSlot::Heiti
        );
        assert_eq!(
            BodyBlock::heading(2, "（一）", "x").font_slot(),
            FontSlot::Kaiti
        );
        assert_eq!(BodyBlock::heading(3, "1.", "x").font_slot(), FontSlot::Body);
        assert_eq!(
            BodyBlock::heading(4, "（1）", "x").font_slot(),
            FontSlot::Body
        );
        assert_eq!(BodyBlock::paragraph("x").font_slot(), FontSlot::Body);
    }

    #[test]
    fn validation_rejects_bad_level() {
        let mut document = example_document();
        document.body.push(BodyBlock {
            level: 5,
            num: Some("x".into()),
            text: "x".into(),
        });
        let diagnostics = document.validate();
        assert!(GbDocument::has_errors(&diagnostics));
    }

    #[test]
    fn validation_warns_upward_without_signer() {
        let mut document = example_document();
        document.doc_kind = DocKind::QingShi;
        let diagnostics = document.validate();
        assert!(diagnostics.iter().any(|d| d.field == "signers"));
    }

    #[test]
    fn json_parse_reports_unknown_doc_kind() {
        let value = json::parse(r#"{"doc_kind":"不存在","issuer":{"names":["x"]},"title":"t","signature":{"date":"2012-07-01"}}"#).unwrap();
        let error = GbDocument::from_json(&value).unwrap_err();
        assert!(error.contains("doc_kind"));
    }

    #[test]
    fn metadata_uses_standard_values() {
        use crate::gbt9704::STANDARD_ID;
        use crate::gbt9704::geometry::{BLACK, Color};
        let document = example_document();
        assert!(document.layout.is_standard());
        assert_eq!(STANDARD_ID, "GB/T 9704-2012");
        let color: Color = BLACK;
        assert_eq!(color.hex(), "#000000");
    }
}

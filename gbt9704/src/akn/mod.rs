//! Akoma Ntoso 3.1 (OASIS LegalDocML v2.0) 序列化器.
//!
//! 设计要点:
//! - 语义内容放进 AKN 原生结构: `meta/identification/FRBR*`、`preface`、`mainBody`、
//!   `conclusions`、`attachments/attachment/doc`、`classification`、`publication`、
//!   `workflow`、`references`、`notes`;
//! - AKN 没有原生对应物的版式信息(版心、字体、位置、页码、版记、印章、特定格式)放进
//!   `meta/proprietary` 与 `meta/presentation` 的扩展命名空间 `urn:gbt9704:ext:1.0`;
//! - 这样 AKN 文件本身就是完整的中间状态: 语义可被 AKN 工具读取, 版式可被本项目
//!   的阶段 2(PDF)无损重建.
//!
//! 命名空间依据: OASIS "Akoma Ntoso Version 2.0, Part 2: Specifications of AKN 3.1",
//! OASIS Standard (2026-10-02 生效), namespace URI 为
//! `http://docs.oasis-open.org/legaldocml/ns/akn/3.1`, 根元素需带 `version="3.1"`.
//!
//! 坑:
//! - `doc` 内所有 `eId` 必须唯一(包括嵌套在附件里的 doc), 这里用 `IdGen` 统一编号;
//! - `meta` 子元素顺序被 schema 固定: identification → publication → classification →
//!   lifecycle → workflow → analysis → temporalData → references → notes → proprietary →
//!   presentation;
//! - `conclusions` 在 `mainBody` 之后、`attachments` 之前, 因此版记(位于附件之后)只能
//!   放进 extension;
//! - `mainBody` 的 `maincontent` 至少需要一个子元素, 正文为空时补一个空 `<p>`.

pub mod xml;

use crate::gbt9704::{self, elements, fonts, geometry};
use crate::model::{Attachment, Attendance, Diagnostic, GbDocument, Severity, Signature};
use xml::{XmlWriter, escape_text, inline_text};

/// AKN 3.0 命名空间(官方 v2.0/AKN 3.1 XSD 实际使用的 targetNamespace).
pub const NS_AKN_30: &str = "http://docs.oasis-open.org/legaldocml/ns/akn/3.0";
/// AKN 3.1 命名空间(OASIS v2.0 命名空间文档声明, 官方未随发对应 XSD).
pub const NS_AKN_31: &str = "http://docs.oasis-open.org/legaldocml/ns/akn/3.1";
/// 默认命名空间: 与官方 XSD 一致, 保证 xmllint 可直接校验.
pub const NS_AKN: &str = NS_AKN_30;
/// AKN 版本号(根元素 `version` 属性的固定值).
pub const AKN_VERSION: &str = "3.1";
/// 本项目扩展命名空间, 只用于 `proprietary`/`presentation` 内部.
pub const NS_GBT: &str = "urn:gbt9704:ext:1.0";
/// 扩展命名空间版本.
pub const NS_GBT_VERSION: &str = "1.0";
/// 元数据来源 agent 的 eId 引用(在 `references` 中定义).
const SOURCE_REF: &str = "#gbt9704_source";
/// 生成器标识, 作为扩展元数据的 source/href.
const GENERATOR_HREF: &str = "https://example.org/gbt9704";

/// eId 生成器: 保证整份文档(含嵌套附件) eId 唯一.
#[derive(Default)]
struct IdGen {
    counters: std::collections::HashMap<String, usize>,
}

impl IdGen {
    fn next(&mut self, prefix: &str) -> String {
        let counter = self.counters.entry(prefix.to_string()).or_insert(0);
        *counter += 1;
        format!("gbt_{prefix}_{counter}")
    }
}

/// 命名空间口径.
///
/// 官方 v2.0/AKN 3.1 包存在内部矛盾: 命名空间文档写 `akn/3.1`,
/// 但唯一发布的 XSD (`akomantoso30.xsd`) 的 targetNamespace 仍是 `akn/3.0`,
/// 只是根元素 `@version` 固定为 `3.1`. 两种口径的元素/属性集合完全相同.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NsMode {
    /// 口径 A: `.../ns/akn/3.0` + `version="3.1"`, 可用官方 XSD 校验(默认).
    Official30,
    /// 口径 B: `.../ns/akn/3.1` + `version="3.1"`, 贴合命名空间文档, 需本地适配 XSD.
    Namespace31,
}

impl NsMode {
    pub fn namespace(self) -> &'static str {
        match self {
            NsMode::Official30 => NS_AKN_30,
            NsMode::Namespace31 => NS_AKN_31,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            NsMode::Official30 => "3.0",
            NsMode::Namespace31 => "3.1",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id.trim() {
            "3.0" | "30" | "akn/3.0" => Some(NsMode::Official30),
            "3.1" | "31" | "akn/3.1" => Some(NsMode::Namespace31),
            _ => None,
        }
    }

    /// 对应 schema 文件名(位于 `schemas/akn31/`).
    pub fn schema_file(self) -> &'static str {
        match self {
            NsMode::Official30 => "akomantoso30.xsd",
            NsMode::Namespace31 => "akomantoso31-adapted.xsd",
        }
    }
}

/// 序列化公文为 AKN 3.1 XML(默认口径 A).
///
/// 返回 `Err(diagnostics)` 表示存在阻断性错误; 警告不会阻断, 由调用方打印.
pub fn serialize(document: &GbDocument) -> Result<String, Vec<Diagnostic>> {
    serialize_with(document, NsMode::Official30)
}

/// 按指定命名空间口径序列化.
pub fn serialize_with(document: &GbDocument, mode: NsMode) -> Result<String, Vec<Diagnostic>> {
    let diagnostics = document.validate();
    if GbDocument::has_errors(&diagnostics) {
        return Err(diagnostics);
    }
    Ok(serialize_unchecked(document, mode))
}

fn serialize_unchecked(document: &GbDocument, mode: NsMode) -> String {
    let mut ids = IdGen::default();
    let mut writer = XmlWriter::new();
    writer.declaration();
    writer.comment("GB/T 9704-2012 党政机关公文格式 -> Akoma Ntoso 3.1 (OASIS LegalDocML v2.0)");
    writer.comment(&format!("语义内容使用 AKN 原生元素; 版式/版记/印章等扩展在 akn:proprietary 与 akn:presentation 中; namespace 口径={}", mode.id()));
    writer.start(
        "akn:akomaNtoso",
        &[
            ("xmlns:akn", mode.namespace()),
            ("xmlns:gbt", NS_GBT),
            ("version", AKN_VERSION),
        ],
    );
    let doc_name = format!("gbt9704-{}", document.doc_kind.slug());
    writer.start("akn:doc", &[("name", doc_name.as_str())]);
    write_meta(&mut writer, document, &mut ids);
    write_preface(&mut writer, document, &mut ids);
    write_main_body(&mut writer, document, &mut ids);
    write_conclusions(&mut writer, document, &mut ids);
    write_attachments(&mut writer, document, &mut ids);
    writer.end("akn:doc");
    writer.end("akn:akomaNtoso");
    writer.finish()
}

// ---------------------------------------------------------------------------
// meta
// ---------------------------------------------------------------------------

fn write_meta(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    writer.start("akn:meta", &[]);
    write_identification(writer, document);
    if let Some(printing) = &document.printing {
        write_publication(writer, printing);
    }
    write_classification(writer, document);
    write_workflow(writer, document, ids);
    write_references(writer, document);
    write_notes(writer, document, ids);
    write_proprietary(writer, document);
    write_presentation(writer, document);
    writer.end("akn:meta");
}

fn write_identification(writer: &mut XmlWriter, document: &GbDocument) {
    let base = document.frbr_base_iri();
    let expression = format!("{base}@");
    let manifestation = format!("{base}.xml");
    let date = document.signature.date.as_str();
    let author = "#gbt_org_1";

    writer.start("akn:identification", &[("source", SOURCE_REF)]);
    // FRBR Work
    writer.start("akn:FRBRWork", &[]);
    writer.empty("akn:FRBRthis", &[("value", &base)]);
    writer.empty("akn:FRBRuri", &[("value", &base)]);
    write_frbr_date(writer, date, "成文日期");
    writer.empty("akn:FRBRauthor", &[("href", author)]);
    writer.empty("akn:FRBRcountry", &[("value", "cn")]);
    writer.empty("akn:FRBRsubtype", &[("value", document.doc_kind.slug())]);
    if let Some(number) = &document.doc_number {
        writer.empty("akn:FRBRnumber", &[("value", &number.display())]);
    }
    writer.empty("akn:FRBRname", &[("value", &document.title)]);
    writer.empty("akn:FRBRprescriptive", &[("value", "true")]);
    writer.end("akn:FRBRWork");

    // FRBR Expression
    writer.start("akn:FRBRExpression", &[]);
    writer.empty("akn:FRBRthis", &[("value", &expression)]);
    writer.empty("akn:FRBRuri", &[("value", &expression)]);
    write_frbr_date(writer, date, "成文日期");
    writer.empty("akn:FRBRauthor", &[("href", author)]);
    writer.empty("akn:FRBRlanguage", &[("language", "zho")]);
    writer.end("akn:FRBRExpression");

    // FRBR Manifestation
    writer.start("akn:FRBRManifestation", &[]);
    writer.empty("akn:FRBRthis", &[("value", &manifestation)]);
    writer.empty("akn:FRBRuri", &[("value", &manifestation)]);
    write_frbr_date(writer, date, "成文日期");
    writer.empty("akn:FRBRauthor", &[("href", author)]);
    writer.empty("akn:FRBRformat", &[("value", "application/xml")]);
    writer.end("akn:FRBRManifestation");
    writer.end("akn:identification");
}

fn write_frbr_date(writer: &mut XmlWriter, date: &str, name: &str) {
    writer.empty("akn:FRBRdate", &[("date", date), ("name", name)]);
}

/// 7.4.3 印发机关和印发日期 → `publication`(官方公布事件, 语义近似).
fn write_publication(writer: &mut XmlWriter, printing: &crate::model::Printing) {
    writer.empty(
        "akn:publication",
        &[
            ("name", printing.issuer.as_str()),
            ("date", printing.date.as_str()),
        ],
    );
}

/// 密级/保密期限与紧急程度 → `classification/keyword`(语义近似).
fn write_classification(writer: &mut XmlWriter, document: &GbDocument) {
    if let Some(secrecy) = &document.secrecy {
        writer.start("akn:classification", &[("source", SOURCE_REF)]);
        let display = secrecy.display();
        writer.empty(
            "akn:keyword",
            &[("dictionary", "#gbt_dict_secrecy"), ("value", &display)],
        );
        writer.end("akn:classification");
    }
    if let Some(urgency) = &document.urgency {
        writer.start("akn:classification", &[("source", SOURCE_REF)]);
        writer.empty(
            "akn:keyword",
            &[
                ("dictionary", "#gbt_dict_urgency"),
                ("value", urgency.as_str()),
            ],
        );
        writer.end("akn:classification");
    }
}

/// 签发人 → `workflow/step`(date/by/as).
fn write_workflow(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    if document.signers.is_empty() {
        return;
    }
    writer.start("akn:workflow", &[("source", SOURCE_REF)]);
    for (index, signer) in document.signers.iter().enumerate() {
        let eid = ids.next("workflow_step");
        let person = format!("#gbt_person_{}", index + 1);
        writer.empty(
            "akn:step",
            &[
                ("eId", eid.as_str()),
                ("date", document.signature.date.as_str()),
                ("by", person.as_str()),
                ("as", "#gbt_role_signatory"),
                ("gbt:role", "签发人"),
                ("gbt:showAs", signer.name.as_str()),
            ],
        );
    }
    writer.end("akn:workflow");
}

/// 发文机关/签发人/字典/角色 → `references` 中的 TLC 元素.
fn write_references(writer: &mut XmlWriter, document: &GbDocument) {
    writer.start("akn:references", &[("source", SOURCE_REF)]);
    // 生成器自身, 供各 source="#gbt9704_source" 引用.
    writer.empty(
        "akn:TLCOrganization",
        &[
            ("eId", "gbt9704_source"),
            ("href", GENERATOR_HREF),
            ("showAs", "GB/T 9704-2012 生成器"),
        ],
    );
    for (index, name) in document.issuer.names.iter().enumerate() {
        let eid = format!("gbt_org_{}", index + 1);
        let href = format!("/akn/cn/organization/gbt9704/{}", index + 1);
        writer.empty(
            "akn:TLCOrganization",
            &[
                ("eId", eid.as_str()),
                ("href", href.as_str()),
                ("showAs", name.as_str()),
            ],
        );
    }
    for (index, signer) in document.signers.iter().enumerate() {
        let eid = format!("gbt_person_{}", index + 1);
        let href = format!("/akn/cn/person/gbt9704/{}", index + 1);
        writer.empty(
            "akn:TLCPerson",
            &[
                ("eId", eid.as_str()),
                ("href", href.as_str()),
                ("showAs", signer.name.as_str()),
            ],
        );
    }
    writer.empty(
        "akn:TLCRole",
        &[
            ("eId", "gbt_role_author"),
            ("href", "/akn/cn/role/author"),
            ("showAs", "发文机关"),
        ],
    );
    writer.empty(
        "akn:TLCRole",
        &[
            ("eId", "gbt_role_signatory"),
            ("href", "/akn/cn/role/signatory"),
            ("showAs", "签发人"),
        ],
    );
    writer.empty(
        "akn:TLCConcept",
        &[
            ("eId", "gbt_dict_secrecy"),
            ("href", "/akn/cn/concept/secrecy"),
            ("showAs", "密级和保密期限"),
        ],
    );
    writer.empty(
        "akn:TLCConcept",
        &[
            ("eId", "gbt_dict_urgency"),
            ("href", "/akn/cn/concept/urgency"),
            ("showAs", "紧急程度"),
        ],
    );
    writer.end("akn:references");
}

/// 7.3.6 附注 → `notes/note`.
fn write_notes(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    let Some(note) = &document.note else {
        return;
    };
    writer.start("akn:notes", &[("source", SOURCE_REF)]);
    let eid = ids.next("note");
    writer.start(
        "akn:note",
        &[
            ("eId", eid.as_str()),
            ("marker", "附注"),
            ("placement", "bottom"),
        ],
    );
    writer.text_element("akn:p", &[], note);
    writer.end("akn:note");
    writer.end("akn:notes");
}

/// 版记等 AKN 无原生元素的元数据 → `proprietary` 扩展.
fn write_proprietary(writer: &mut XmlWriter, document: &GbDocument) {
    writer.start("akn:proprietary", &[("source", SOURCE_REF)]);
    writer.start(
        "gbt:gbt9704",
        &[
            ("version", NS_GBT_VERSION),
            ("standard", gbt9704::STANDARD_ID),
        ],
    );
    let has_record =
        !document.cc.is_empty() || document.printing.is_some() || document.addressees_in_record;
    if has_record {
        writer.start("gbt:record", &[]);
        if document.addressees_in_record {
            for addressee in &document.addressees {
                writer.text_element("gbt:addresseeMoved", &[], addressee);
            }
        }
        for cc in &document.cc {
            writer.text_element("gbt:cc", &[], cc);
        }
        if let Some(printing) = &document.printing {
            let text = format!(
                "{} {}印发",
                printing.issuer,
                crate::model::iso_date_to_zh(&printing.date)
                    .unwrap_or_else(|| printing.date.clone())
            );
            writer.text_element(
                "gbt:printing",
                &[
                    ("issuer", printing.issuer.as_str()),
                    ("date", printing.date.as_str()),
                ],
                &text,
            );
        }
        writer.end("gbt:record");
    }
    if let Some(order_number) = &document.order_number {
        writer.text_element("gbt:orderNumber", &[], order_number);
    }
    writer.end("gbt:gbt9704");
    writer.end("akn:proprietary");
}

/// 版式/字体/页码/版记规格/特定格式参数 → `presentation` 扩展.
///
/// 这里把 `elements::ELEMENTS` 全量写入: 标准中每一个格式要素都在 AKN 里有一份
/// 机器可读的呈现规格, 保证"中间状态 = Akoma Ntoso"是完整的.
fn write_presentation(writer: &mut XmlWriter, document: &GbDocument) {
    let layout = &document.layout;
    let (paper_width, paper_height) = layout.paper_size_mm();

    writer.start("akn:presentation", &[("source", SOURCE_REF)]);
    writer.start(
        "gbt:presentation",
        &[
            ("version", NS_GBT_VERSION),
            ("standard", gbt9704::STANDARD_ID),
            ("standard-name", gbt9704::STANDARD_NAME),
            ("issued", gbt9704::STANDARD_ISSUED),
            ("effective", gbt9704::STANDARD_EFFECTIVE),
        ],
    );

    let paper_width = format!("{paper_width}");
    let paper_height = format!("{paper_height}");
    writer.empty(
        "gbt:page",
        &[("width-mm", &paper_width), ("height-mm", &paper_height)],
    );

    let margin_top = format!("{:.4}", layout.margin_top_mm);
    let margin_left = format!("{:.4}", layout.margin_left_mm);
    let margin_right = format!("{:.4}", layout.margin_right_mm());
    let margin_bottom = format!("{:.4}", layout.margin_bottom_mm());
    writer.empty(
        "gbt:margins",
        &[
            ("top-mm", &margin_top),
            ("left-mm", &margin_left),
            ("right-mm", &margin_right),
            ("bottom-mm", &margin_bottom),
            ("top-tolerance-mm", "1"),
            ("left-tolerance-mm", "1"),
        ],
    );

    let text_width = format!("{:.4}", layout.text_width_mm);
    let text_height = format!("{:.4}", layout.text_height_mm);
    writer.empty(
        "gbt:text-area",
        &[
            ("width-mm", &text_width),
            ("height-mm", &text_height),
            ("chars-per-line", "28"),
            ("lines-per-page", "22"),
        ],
    );

    let char_pitch = format!("{:.4}", layout.char_pitch_mm);
    let line_pitch = format!("{:.4}", layout.line_pitch_mm);
    let char_pitch_pt = format!(
        "{:.4}",
        layout.char_pitch_mm * geometry::PT_PER_INCH / geometry::MM_PER_INCH
    );
    let line_pitch_pt = format!(
        "{:.4}",
        layout.line_pitch_mm * geometry::PT_PER_INCH / geometry::MM_PER_INCH
    );
    let line_unit_pt = format!("{:.4}", geometry::line_unit_pt());
    writer.empty(
        "gbt:grid",
        &[
            ("char-pitch-mm", &char_pitch),
            ("line-pitch-mm", &line_pitch),
            ("char-pitch-pt", &char_pitch_pt),
            ("line-pitch-pt", &line_pitch_pt),
            ("line-unit-pt", &line_unit_pt),
        ],
    );

    let black = geometry::BLACK.hex();
    let red = geometry::RED.hex();
    let (c, m, y, k) = geometry::RED_CMYK;
    let red_cmyk = format!("{c},{m},{y},{k}");
    writer.empty(
        "gbt:colors",
        &[("black", &black), ("red", &red), ("red-cmyk", &red_cmyk)],
    );

    // 字体槽位.
    for requirement in fonts::REQUIREMENTS {
        let slot = requirement.slot.id();
        let family = requirement.slot.standard_family();
        let size = requirement
            .size_pt
            .map(|pt| format!("{pt}"))
            .unwrap_or_else(|| "auto".to_string());
        let color = requirement.color.hex();
        let usage = requirement.usage;
        let clause = requirement.clause;
        writer.empty(
            "gbt:font",
            &[
                ("slot", slot),
                ("family", family),
                ("size-pt", &size),
                ("color", &color),
                ("usage", usage),
                ("clause", clause),
            ],
        );
    }

    // 每个标准要素的呈现规格.
    writer.start("gbt:elements", &[]);
    for spec in elements::ELEMENTS {
        let size = spec
            .font_size_pt
            .map(|pt| format!("{pt}"))
            .unwrap_or_default();
        let font = spec.font.map(|slot| slot.id()).unwrap_or("none");
        let color = spec.color.hex();
        writer.text_element(
            "gbt:element",
            &[
                ("id", spec.id),
                ("part", spec.part.id()),
                ("requirement", spec.requirement.id()),
                ("font", font),
                ("size-pt", &size),
                ("color", &color),
                ("align", spec.align.id()),
                ("position", spec.position),
                ("spacing", spec.spacing),
                ("clause", spec.clause),
            ],
            spec.rules,
        );
    }
    writer.end("gbt:elements");

    // 页码(7.5).
    let page_font = format!("{}", geometry::PAGE_NUMBER_FONT_PT);
    writer.empty(
        "gbt:page-number",
        &[
            (
                "enabled",
                if document.page.enabled {
                    "true"
                } else {
                    "false"
                },
            ),
            ("offset-mm", "7"),
            ("side-blank-chars", "1"),
            ("dash-width-mm", "4.9389"),
            ("font", "page_number"),
            ("size-pt", &page_font),
            ("continuous-with-attachments", "true"),
            ("blank-and-record-pages", "false"),
        ],
    );

    // 发文机关标志的字号标准未规定; 给出渲染默认/范围(式样图图测 33–39 pt).
    let issuer_default = format!("{}", geometry::ISSUER_MARK_DEFAULT_PT);
    let issuer_min = format!("{}", geometry::ISSUER_MARK_MIN_PT);
    let issuer_max = format!("{}", geometry::ISSUER_MARK_MAX_PT);
    writer.empty(
        "gbt:issuer-mark",
        &[
            ("default-size-pt", &issuer_default),
            ("min-size-pt", &issuer_min),
            ("max-size-pt", &issuer_max),
            ("color", &red),
            ("join-standard", "文件"),
        ],
    );

    // 版记分隔线(7.4.1).
    let thick = format!("{}", geometry::RECORD_LINE_THICK_MM);
    let thin = format!("{}", geometry::RECORD_LINE_THIN_MM);
    writer.empty(
        "gbt:record-lines",
        &[
            ("width", "text-area"),
            ("first-thickness-mm", &thick),
            ("last-thickness-mm", &thick),
            ("inner-thickness-mm", &thin),
        ],
    );

    // 特定格式参数(第 10 章).
    match document.format {
        crate::model::Format::Standard => {
            writer.empty("gbt:format", &[("name", "standard")]);
        }
        crate::model::Format::Letter => {
            writer.empty(
                "gbt:format",
                &[
                    ("name", "letter"),
                    ("issuer-mark-top-from-paper-mm", "30"),
                    ("issuer-mark-color", &red),
                    ("red-line-top-gap-mm", "4"),
                    ("red-line-top-style", "thick-over-thin"),
                    ("red-line-bottom-gap-mm", "20"),
                    ("red-line-bottom-style", "thin-over-thick"),
                    ("red-line-width-mm", "170"),
                    ("red-line-centered", "true"),
                    ("first-page-page-number", "false"),
                    ("record-without-printing", "true"),
                    ("record-without-separator-lines", "true"),
                    ("record-at-bottom", "true"),
                ],
            );
        }
        crate::model::Format::Order => {
            writer.empty(
                "gbt:format",
                &[
                    ("name", "order"),
                    ("issuer-mark-top-from-text-mm", "20"),
                    ("issuer-mark-join", "命令"),
                    ("order-number-gap-lines", "2"),
                    ("body-gap-lines", "2"),
                ],
            );
        }
        crate::model::Format::Minutes => {
            writer.empty(
                "gbt:format",
                &[
                    ("name", "minutes"),
                    ("issuer-mark-top-from-text-mm", "35"),
                    ("issuer-mark-join", "纪要"),
                    ("attendance-gap-lines", "1"),
                    ("attendance-indent-chars", "2"),
                ],
            );
        }
    }

    writer.end("gbt:presentation");
    writer.end("akn:presentation");
}

// ---------------------------------------------------------------------------
// preface: 版头 + 标题 + 主送机关(可见内容)
// ---------------------------------------------------------------------------

fn write_preface(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    writer.start("akn:preface", &[("eId", "gbt_preface")]);

    // 信函格式: 发文机关标志在上, 红色双线下才是份号/密级/紧急程度/发文字号(10.1).
    let header_first = document.format != crate::model::Format::Letter;
    if header_first {
        write_header_lines(writer, document, ids);
    }

    let mark = document.issuer.mark_text_for(document.format);
    preface_text(
        writer,
        ids,
        "issuerMark",
        &[("gbt:format", document.format.id())],
        &mark,
    );

    if !header_first {
        write_header_lines(writer, document, ids);
    }

    write_doc_number(writer, document, ids);

    if !document.signers.is_empty() {
        let names = document
            .signers
            .iter()
            .map(|signer| signer.name.as_str())
            .collect::<Vec<_>>()
            .join("、");
        let inner = format!("签发人：{}", inline_text("akn:docProponent", &[], &names));
        preface_raw(writer, ids, "signers", &[], &inner);
    }

    preface_raw(
        writer,
        ids,
        "title",
        &[],
        &inline_text("akn:docTitle", &[], &document.title),
    );

    // 文种: AKN 的 docType 是自由文本 inline 元素; 公文标题已含文种, 这里存机读值,
    // 渲染层按 gbt:render="false" 跳过, 避免在版面上重复显示.
    preface_raw(
        writer,
        ids,
        "docKind",
        &[("gbt:render", "false")],
        &inline_text("akn:docType", &[], document.doc_kind.zh()),
    );

    if !document.addressees.is_empty() && !document.addressees_in_record {
        let text = format!("{}：", document.addressees.join("，"));
        preface_text(writer, ids, "addressee", &[], &text);
    }

    writer.end("akn:preface");
}

/// 份号/密级和保密期限/紧急程度: 顶格, 按顺序自上而下(7.2.1~7.2.3).
fn write_header_lines(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    if let Some(copy_number) = &document.copy_number {
        preface_text(
            writer,
            ids,
            "copyNumber",
            &[("gbt:digits", "6")],
            copy_number,
        );
    }
    if let Some(secrecy) = &document.secrecy {
        let display = secrecy.display();
        let level = secrecy.level.zh();
        let term = secrecy.term.as_deref().unwrap_or("");
        preface_text(
            writer,
            ids,
            "secrecy",
            &[("gbt:level", level), ("gbt:term", term)],
            &display,
        );
    }
    if let Some(urgency) = &document.urgency {
        preface_text(writer, ids, "urgency", &[], urgency);
    }
}

/// 发文字号(7.2.5) / 命令(令)的令号(10.2).
fn write_doc_number(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    let (role, value) = match document.format {
        crate::model::Format::Order => {
            let value = document
                .order_number
                .clone()
                .or_else(|| {
                    document
                        .doc_number
                        .as_ref()
                        .map(|n| format!("第{}号", n.serial))
                })
                .unwrap_or_default();
            ("orderNumber", value)
        }
        _ => match &document.doc_number {
            Some(number) => ("docNumber", number.display()),
            None => return,
        },
    };
    if value.is_empty() {
        return;
    }
    preface_raw(
        writer,
        ids,
        role,
        &[],
        &inline_text("akn:docNumber", &[], &value),
    );
}

fn preface_text(
    writer: &mut XmlWriter,
    ids: &mut IdGen,
    role: &str,
    extra_attrs: &[(&str, &str)],
    text: &str,
) {
    preface_raw(writer, ids, role, extra_attrs, &escape_text(text));
}

fn preface_raw(
    writer: &mut XmlWriter,
    ids: &mut IdGen,
    role: &str,
    extra_attrs: &[(&str, &str)],
    inner_xml: &str,
) {
    role_p(writer, ids, "preface", role, extra_attrs, inner_xml);
}

/// 生成一个带 `gbt:role` 的段落; prefix 只影响 eId 的可读性, 不参与语义.
fn role_p(
    writer: &mut XmlWriter,
    ids: &mut IdGen,
    prefix: &str,
    role: &str,
    extra_attrs: &[(&str, &str)],
    inner_xml: &str,
) {
    let eid = ids.next(prefix);
    let mut attrs: Vec<(&str, &str)> = vec![("eId", eid.as_str()), ("gbt:role", role)];
    attrs.extend_from_slice(extra_attrs);
    writer.raw_element("akn:p", &attrs, inner_xml);
}

// ---------------------------------------------------------------------------
// mainBody: 正文 + 结构层次序数
// ---------------------------------------------------------------------------

/// 正文树节点: 保留原始顺序, 层次标题可嵌套任意层(7.3.3 的 4 层).
enum BodyItem {
    Paragraph(String),
    Heading {
        level: u8,
        num: String,
        text: String,
        items: Vec<BodyItem>,
    },
}

/// 把平铺的正文块解析成树.
///
/// 规则: 层号更大的块归入前一个层号更小的标题; 自然段(level=0)归入当前最近的标题.
fn build_body_tree(blocks: &[crate::model::BodyBlock]) -> Vec<BodyItem> {
    let mut index = 0;
    build_items(blocks, &mut index, 0)
}

fn build_items(
    blocks: &[crate::model::BodyBlock],
    index: &mut usize,
    parent_level: u8,
) -> Vec<BodyItem> {
    let mut items = Vec::new();
    while *index < blocks.len() {
        let block = &blocks[*index];
        if block.level == 0 {
            items.push(BodyItem::Paragraph(block.text.clone()));
            *index += 1;
            continue;
        }
        if block.level <= parent_level {
            break;
        }
        *index += 1;
        let children = build_items(blocks, index, block.level);
        items.push(BodyItem::Heading {
            level: block.level,
            num: block.num.clone().unwrap_or_default(),
            text: block.text.clone(),
            items: children,
        });
    }
    items
}

fn write_main_body(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    writer.start("akn:mainBody", &[("eId", "gbt_main_body")]);
    if document.body.is_empty() {
        // maincontent 至少要求一个子元素; 空正文用空 p 占位(校验已给出警告).
        writer.empty("akn:p", &[("eId", "gbt_body_empty")]);
    } else {
        let tree = build_body_tree(&document.body);
        write_body_items(writer, &tree, ids, "body");
    }
    writer.end("akn:mainBody");
}

fn write_body_items(writer: &mut XmlWriter, items: &[BodyItem], ids: &mut IdGen, prefix: &str) {
    for item in items {
        match item {
            BodyItem::Paragraph(text) => {
                let eid = ids.next(prefix);
                writer.text_element("akn:p", &[("eId", eid.as_str())], text);
            }
            BodyItem::Heading {
                level,
                num,
                text,
                items: children,
            } => {
                let eid = ids.next(prefix);
                let level_attr = level.to_string();
                let name = format!("level-{level}");
                writer.start(
                    "akn:hcontainer",
                    &[
                        ("eId", eid.as_str()),
                        ("name", name.as_str()),
                        ("gbt:level", level_attr.as_str()),
                    ],
                );
                if !num.is_empty() {
                    writer.text_element("akn:num", &[], num);
                }
                writer.text_element("akn:heading", &[], text);
                if children.is_empty() {
                    writer.start("akn:content", &[]);
                    writer.end("akn:content");
                } else {
                    let (leading, rest) = split_leading_paragraphs(children);
                    let trailing = trailing_paragraphs(rest);
                    let headings = &rest[..rest.len() - trailing.len()];
                    if !leading.is_empty() {
                        writer.start("akn:intro", &[]);
                        write_body_items(writer, leading, ids, prefix);
                        writer.end("akn:intro");
                    }
                    write_body_items(writer, headings, ids, prefix);
                    if !trailing.is_empty() {
                        writer.start("akn:wrapUp", &[]);
                        write_body_items(writer, trailing, ids, prefix);
                        writer.end("akn:wrapUp");
                    }
                }
                writer.end("akn:hcontainer");
            }
        }
    }
}

fn split_leading_paragraphs(items: &[BodyItem]) -> (&[BodyItem], &[BodyItem]) {
    let first_heading = items
        .iter()
        .position(|item| matches!(item, BodyItem::Heading { .. }))
        .unwrap_or(items.len());
    items.split_at(first_heading)
}

fn trailing_paragraphs(items: &[BodyItem]) -> &[BodyItem] {
    if let Some(last_heading) = items
        .iter()
        .rposition(|item| matches!(item, BodyItem::Heading { .. }))
    {
        &items[last_heading + 1..]
    } else {
        &[]
    }
}

// ---------------------------------------------------------------------------
// conclusions: 附件说明 + 出席名单 + 署名 + 成文日期 + 印章 + 附注
// ---------------------------------------------------------------------------

fn write_conclusions(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    writer.start("akn:conclusions", &[("eId", "gbt_conclusions")]);

    // 7.3.4 附件说明.
    for (index, note) in document.attachment_notes.iter().enumerate() {
        let prefix = if index == 0 { "附件：" } else { "" };
        let text = format!("{prefix}{}", note.display());
        let serial = note.serial.map(|s| s.to_string()).unwrap_or_default();
        let eid = ids.next("conclusion");
        writer.text_element(
            "akn:p",
            &[
                ("eId", eid.as_str()),
                ("gbt:role", "attachmentNote"),
                ("gbt:serial", serial.as_str()),
                ("gbt:showAs", note.name.as_str()),
            ],
            &text,
        );
    }

    // 10.3 纪要出席/请假/列席人员名单.
    if let Some(attendance) = &document.attendance {
        write_attendance(writer, attendance, ids);
    }

    // 7.3.5 发文机关署名(联合行文按顺序逐行).
    let signature = &document.signature;
    for issuer in document.signature_issuer_names() {
        let eid = ids.next("conclusion");
        writer.raw_element(
            "akn:p",
            &[("eId", eid.as_str()), ("gbt:role", "issuerSignature")],
            &inline_text("akn:signature", &[], &issuer),
        );
    }

    // 8.3.5.4 成文日期.
    let date_zh = document.date_zh();
    let eid = ids.next("conclusion");
    writer.raw_element(
        "akn:p",
        &[("eId", eid.as_str()), ("gbt:role", "documentDate")],
        &inline_text("akn:date", &[("date", signature.date.as_str())], &date_zh),
    );

    // 印章/签名章: 图形资源, 在内容里留占位并在属性中携带元数据.
    write_seals(writer, signature, ids);

    // 7.3.6 附注: 可见内容.
    if let Some(note) = &document.note {
        role_p(
            writer,
            ids,
            "conclusion",
            "annotation",
            &[],
            &escape_text(note),
        );
    }

    writer.end("akn:conclusions");
}

fn write_attendance(writer: &mut XmlWriter, attendance: &Attendance, ids: &mut IdGen) {
    let groups = [
        ("present", "出席", &attendance.present),
        ("absent", "请假", &attendance.absent),
        ("observers", "列席", &attendance.observers),
    ];
    for (kind, label, members) in groups {
        if members.is_empty() {
            continue;
        }
        let eid = ids.next("conclusion");
        let inner = format!(
            "{}<akn:span gbt:role=\"attendanceNames\">{}</akn:span>",
            escape_text(&format!("{label}：")),
            escape_text(&members.join("、"))
        );
        writer.raw_element(
            "akn:p",
            &[
                ("eId", eid.as_str()),
                ("gbt:role", "attendance"),
                ("gbt:kind", kind),
            ],
            &inner,
        );
    }
}

fn write_seals(writer: &mut XmlWriter, signature: &Signature, ids: &mut IdGen) {
    for seal in &signature.seals {
        let eid = ids.next("seal");
        let mut attrs: Vec<(&str, &str)> = vec![
            ("eId", eid.as_str()),
            ("gbt:role", "seal"),
            ("gbt:mode", signature.mode.id()),
        ];
        if let Some(image) = &seal.image {
            attrs.push(("gbt:image", image.as_str()));
        }
        if let Some(issuer) = &seal.issuer {
            attrs.push(("gbt:issuer", issuer.as_str()));
        }
        if let Some(signer) = &seal.signer {
            attrs.push(("gbt:signer", signer.as_str()));
        }
        if let Some(title) = &seal.title {
            attrs.push(("gbt:title", title.as_str()));
        }
        writer.empty("akn:p", &attrs);
    }
}

// ---------------------------------------------------------------------------
// attachments: 附件实体(另面编排, 在版记之前)
// ---------------------------------------------------------------------------

fn write_attachments(writer: &mut XmlWriter, document: &GbDocument, ids: &mut IdGen) {
    if document.attachments.is_empty() {
        return;
    }
    writer.start("akn:attachments", &[]);
    for attachment in &document.attachments {
        write_attachment(writer, document, attachment, ids);
    }
    writer.end("akn:attachments");
}

fn write_attachment(
    writer: &mut XmlWriter,
    document: &GbDocument,
    attachment: &Attachment,
    ids: &mut IdGen,
) {
    let prefix = format!("att{}", attachment.serial);
    let attachment_eid = ids.next(&format!("attachment_{}", attachment.serial));
    writer.start("akn:attachment", &[("eId", attachment_eid.as_str())]);
    let doc_name = format!("附件{}", attachment.serial);
    writer.start("akn:doc", &[("name", doc_name.as_str())]);

    // 附件文档的最小 identification(独立 FRBR 链, 但仍属于同一份公文).
    let base = format!(
        "{}/attachment/{}",
        document.frbr_base_iri(),
        attachment.serial
    );
    let expression = format!("{base}@");
    let manifestation = format!("{base}.xml");
    let date = document.signature.date.as_str();
    writer.start("akn:meta", &[]);
    writer.start("akn:identification", &[("source", SOURCE_REF)]);
    writer.start("akn:FRBRWork", &[]);
    writer.empty("akn:FRBRthis", &[("value", &base)]);
    writer.empty("akn:FRBRuri", &[("value", &base)]);
    write_frbr_date(writer, date, "成文日期");
    writer.empty("akn:FRBRauthor", &[("href", "#gbt_org_1")]);
    writer.empty("akn:FRBRcountry", &[("value", "cn")]);
    writer.empty("akn:FRBRname", &[("value", &attachment.title)]);
    writer.end("akn:FRBRWork");
    writer.start("akn:FRBRExpression", &[]);
    writer.empty("akn:FRBRthis", &[("value", &expression)]);
    writer.empty("akn:FRBRuri", &[("value", &expression)]);
    write_frbr_date(writer, date, "成文日期");
    writer.empty("akn:FRBRauthor", &[("href", "#gbt_org_1")]);
    writer.empty("akn:FRBRlanguage", &[("language", "zho")]);
    writer.end("akn:FRBRExpression");
    writer.start("akn:FRBRManifestation", &[]);
    writer.empty("akn:FRBRthis", &[("value", &manifestation)]);
    writer.empty("akn:FRBRuri", &[("value", &manifestation)]);
    write_frbr_date(writer, date, "成文日期");
    writer.empty("akn:FRBRauthor", &[("href", "#gbt_org_1")]);
    writer.empty("akn:FRBRformat", &[("value", "application/xml")]);
    writer.end("akn:FRBRManifestation");
    writer.end("akn:identification");
    writer.end("akn:meta");

    let preface_eid = format!("gbt_{prefix}_preface");
    writer.start("akn:preface", &[("eId", preface_eid.as_str())]);
    if attachment.standalone
        && let Some(number) = &document.doc_number
    {
        let label = format!("{}附件{}", number.display(), attachment.serial);
        let eid = ids.next(&prefix);
        writer.text_element(
            "akn:p",
            &[
                ("eId", eid.as_str()),
                ("gbt:role", "attachmentStandaloneLabel"),
            ],
            &label,
        );
    }
    let eid = ids.next(&prefix);
    writer.raw_element(
        "akn:p",
        &[("eId", eid.as_str()), ("gbt:role", "attachmentTitle")],
        &inline_text("akn:docTitle", &[], &attachment.title),
    );
    writer.end("akn:preface");

    let main_body_eid = format!("gbt_{prefix}_main_body");
    writer.start("akn:mainBody", &[("eId", main_body_eid.as_str())]);
    if attachment.paragraphs.is_empty() {
        let eid = ids.next(&prefix);
        writer.empty("akn:p", &[("eId", eid.as_str())]);
    } else {
        for paragraph in &attachment.paragraphs {
            let eid = ids.next(&prefix);
            writer.text_element("akn:p", &[("eId", eid.as_str())], paragraph);
        }
    }
    writer.end("akn:mainBody");

    writer.end("akn:doc");
    writer.end("akn:attachment");
}

/// 供 CLI 打印的序列化摘要.
pub fn summary(document: &GbDocument, xml: &str) -> String {
    format!(
        "文种={} 格式={} 正文块={} 附件={} 字节={}",
        document.doc_kind.zh(),
        document.format.zh(),
        document.body.len(),
        document.attachments.len(),
        xml.len()
    )
}

/// 校验诊断里是否有阻断错误.
pub fn blocks_serialization(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::example_document;

    #[test]
    fn body_tree_nests_headings_by_level() {
        use crate::model::BodyBlock;
        let blocks = vec![
            BodyBlock::paragraph("前言"),
            BodyBlock::heading(1, "一、", "第一层"),
            BodyBlock::paragraph("一层正文"),
            BodyBlock::heading(2, "（一）", "第二层"),
            BodyBlock::paragraph("二层正文"),
            BodyBlock::heading(1, "二、", "另一个第一层"),
        ];
        let tree = build_body_tree(&blocks);
        assert_eq!(tree.len(), 3);
        match &tree[1] {
            BodyItem::Heading {
                level, num, items, ..
            } => {
                assert_eq!(*level, 1);
                assert_eq!(num, "一、");
                assert_eq!(items.len(), 2);
                match &items[1] {
                    BodyItem::Heading { level, .. } => assert_eq!(*level, 2),
                    _ => panic!("第二项应为二级标题"),
                }
            }
            _ => panic!("第二项应为一级标题"),
        }
    }

    #[test]
    fn example_serializes_with_expected_structure() {
        let document = example_document();
        let xml = serialize(&document).expect("示例应可序列化");
        assert!(xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(xml.contains("xmlns:akn=\"http://docs.oasis-open.org/legaldocml/ns/akn/3.0\""));
        assert!(xml.contains("xmlns:gbt=\"urn:gbt9704:ext:1.0\""));
        assert!(xml.contains("version=\"3.1\""));
        assert!(xml.contains("<akn:FRBRWork>"));
        assert!(xml.contains("<akn:docTitle>"));
        assert!(xml.contains("<akn:mainBody"));
        assert!(xml.contains("<akn:hcontainer"));
        assert!(xml.contains("<akn:attachments"));
        assert!(xml.contains("<akn:proprietary"));
        assert!(xml.contains("<akn:presentation"));
        // eId 唯一.
        let eids: Vec<&str> = xml
            .match_indices("eId=\"")
            .map(|(index, _)| {
                let start = index + 5;
                let rest = &xml[start..];
                let end = rest.find('"').unwrap();
                &rest[..end]
            })
            .collect();
        let unique: std::collections::HashSet<&str> = eids.iter().copied().collect();
        assert_eq!(eids.len(), unique.len(), "eId 必须唯一");
    }

    #[test]
    fn ns_mode_3_1_changes_only_namespace() {
        let document = example_document();
        let xml30 = serialize_with(&document, NsMode::Official30).unwrap();
        let xml31 = serialize_with(&document, NsMode::Namespace31).unwrap();
        assert!(xml30.contains("ns/akn/3.0"));
        assert!(xml31.contains("ns/akn/3.1"));
        let normalize = |xml: &str, mode: &str| {
            xml.replace("ns/akn/3.0", "NS")
                .replace("ns/akn/3.1", "NS")
                .replace(mode, "MODE")
        };
        assert_eq!(normalize(&xml30, "口径=3.0"), normalize(&xml31, "口径=3.1"));
    }

    #[test]
    fn special_formats_change_issuer_mark_and_parameters() {
        let mut document = example_document();
        document.format = crate::model::Format::Order;
        let xml = serialize(&document).unwrap();
        assert!(xml.contains("gbt:format") && xml.contains("name=\"order\""));
        assert!(xml.contains("命令"));
        document.format = crate::model::Format::Letter;
        let xml = serialize(&document).unwrap();
        assert!(xml.contains("name=\"letter\""));
        assert!(xml.contains("red-line-width-mm=\"170\""));
    }
}

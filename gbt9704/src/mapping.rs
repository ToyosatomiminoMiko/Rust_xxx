//! GB/T 9704-2012 每一项 → Akoma Ntoso 3.1 的映射表.
//!
//! 映射分四类(见 `MappingKind`), 目标是让"标准里的每一项"都有明确去向:
//! - Direct: AKN 原生元素/属性可精确表达;
//! - NearFit: 有原生元素但语义有偏移, 需要讨论;
//! - Extension: AKN 没有原生元素, 使用官方扩展点 `proprietary`/`presentation`;
//! - NoFit: 纯版式/印制/视觉规则, AKN 语义模型无对应, 由阶段 2 渲染/印后工艺执行,
//!   但呈现参数仍写入 `presentation`, 保证 AKN 文件是完整中间状态.
//!
//! 本表与 `gbt9704::elements::ELEMENTS` 一一对应; `tests/mapping.rs` 会自检覆盖性.

use crate::gbt9704::elements::{self, ElementSpec, Part};
use crate::json::{self, Value};

/// 映射类别.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingKind {
    Direct,
    NearFit,
    Extension,
    NoFit,
}

impl MappingKind {
    pub fn id(self) -> &'static str {
        match self {
            MappingKind::Direct => "direct",
            MappingKind::NearFit => "near_fit",
            MappingKind::Extension => "extension",
            MappingKind::NoFit => "no_fit",
        }
    }

    pub fn zh(self) -> &'static str {
        match self {
            MappingKind::Direct => "直接对应",
            MappingKind::NearFit => "语义近似",
            MappingKind::Extension => "扩展承载",
            MappingKind::NoFit => "无对应(版式/印制)",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            MappingKind::Direct => "AKN 原生元素/属性可精确表达",
            MappingKind::NearFit => "AKN 有相近元素, 但语义有偏移, 需讨论",
            MappingKind::Extension => {
                "AKN 无原生元素, 使用 proprietary/presentation 官方扩展点承载"
            }
            MappingKind::NoFit => "AKN 语义模型无对应, 由渲染/印后工艺执行",
        }
    }
}

/// 一条映射.
#[derive(Debug, Clone, Copy)]
pub struct Mapping {
    /// 对应 `ElementSpec::id`.
    pub gb_id: &'static str,
    /// AKN 侧元素/属性/存放路径.
    pub akn_target: &'static str,
    pub kind: MappingKind,
    /// 数据在 AKN 文件中的具体位置.
    pub storage: &'static str,
    /// 冲突/语义偏移说明; Direct 通常为 None.
    pub conflict: Option<&'static str>,
}

/// 全量映射表: 必须覆盖 `gbt9704::elements::ELEMENTS` 的每一项.
pub const MAPPINGS: &[Mapping] = &[
    // ---------------- 用纸/版面/印制 ----------------
    Mapping {
        gb_id: "paper.technical",
        akn_target: "无 AKN 语义; presentation/gbt:element[paper.technical]",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:presentation/gbt:elements",
        conflict: Some("纸张物理指标是印后/材料要求, 不属于文档语义; 仅作为呈现参数存档"),
    },
    Mapping {
        gb_id: "paper.a4",
        akn_target: "无 AKN 语义; presentation/gbt:page",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:page",
        conflict: Some("幅面是 manifestation 的物理属性; AKN 无页面尺寸元素"),
    },
    Mapping {
        gb_id: "layout.margins",
        akn_target: "无 AKN 语义; presentation/gbt:margins + gbt:text-area",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:margins",
        conflict: Some("页边/版心属于版面几何; AKN 只描述结构, 不描述版心"),
    },
    Mapping {
        gb_id: "layout.default_font",
        akn_target: "无 AKN 语义; presentation/gbt:font[slot=body]",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:font",
        conflict: Some("字体字号是呈现属性; AKN 的文本内容不带字体"),
    },
    Mapping {
        gb_id: "layout.grid",
        akn_target: "无 AKN 语义; presentation/gbt:text-area + gbt:grid",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:grid",
        conflict: Some("22 行×28 字是版面网格; AKN 无行数字数概念"),
    },
    Mapping {
        gb_id: "layout.units",
        akn_target: "无 AKN 语义; presentation/gbt:grid(line-unit-pt/char-pitch-pt) + presentation/gbt:element[layout.units]",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:grid + gbt:elements",
        conflict: Some(
            "“字/行”是排版距离单位; 术语定义值(16 pt/30 pt)与 22 行×28 字网格值(15.79 pt/28.99 pt)是两套常量, AKN 只能存参数",
        ),
    },
    Mapping {
        gb_id: "layout.text_color",
        akn_target: "无 AKN 语义; presentation/gbt:colors",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:colors",
        conflict: Some("文字颜色是呈现属性"),
    },
    Mapping {
        gb_id: "printing.plate",
        akn_target: "无 AKN 语义; presentation/gbt:element[printing.plate]",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:elements",
        conflict: Some("制版质量要求不属于文档内容"),
    },
    Mapping {
        gb_id: "printing.ink",
        akn_target: "无 AKN 语义; presentation/gbt:colors(黑色/红色油墨)",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:colors",
        conflict: Some("印刷油墨色值(CMY K)属于印后工艺; RGB/CMYK 由阶段 2 处理"),
    },
    Mapping {
        gb_id: "printing.binding",
        akn_target: "无 AKN 语义; presentation/gbt:element[printing.binding]",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:elements",
        conflict: Some("装订方式/订位不属于文档结构"),
    },
    // ---------------- 版头 ----------------
    Mapping {
        gb_id: "header.copy_number",
        akn_target: "preface/p[@gbt:role='copyNumber'] + gbt:digits",
        kind: MappingKind::Extension,
        storage: "akn:doc/preface",
        conflict: Some(
            "AKN 无份号元素; 理论上可借用 FRBR Item 层标识同一 expression 的不同副本, 但 AKN 没有 item 级编号元素",
        ),
    },
    Mapping {
        gb_id: "header.secrecy",
        akn_target: "meta/classification/keyword(dictionary=#gbt_dict_secrecy) + preface/p[@gbt:role='secrecy']",
        kind: MappingKind::NearFit,
        storage: "akn:meta/classification + akn:doc/preface",
        conflict: Some(
            "classification/keyword 的语义是主题分类/受控词表, 不是保密等级; 密级和保密期限被当作关键词承载, 语义偏移",
        ),
    },
    Mapping {
        gb_id: "header.urgency",
        akn_target: "meta/classification/keyword(dictionary=#gbt_dict_urgency) + preface/p[@gbt:role='urgency']",
        kind: MappingKind::NearFit,
        storage: "akn:meta/classification + akn:doc/preface",
        conflict: Some("紧急程度同样不是主题关键词; 若 AKN 未来提供 docUrgency 类元素应迁移"),
    },
    Mapping {
        gb_id: "header.issuer_mark",
        akn_target: "FRBRauthor(TLCOrganization) + preface/p[@gbt:role='issuerMark'] + presentation/gbt:font[issuer_mark]",
        kind: MappingKind::NearFit,
        storage: "akn:meta/identification + akn:doc/preface + akn:presentation",
        conflict: Some(
            "发文机关标志 = 机关名称(语义) + 红色小标宋/35 mm 版头位置(视觉); AKN 只表达机关, 视觉部分只能进扩展",
        ),
    },
    Mapping {
        gb_id: "header.joint_issuer_mark",
        akn_target: "多个 FRBRauthor(TLCOrganization) + preface/p[@gbt:role='issuerMark'] + presentation",
        kind: MappingKind::NearFit,
        storage: "akn:meta/identification + akn:doc/preface",
        conflict: Some(
            "联合行文“主办机关在前、文件二字在右、上下居中”是版式规则; AKN 用多个并列 author 表达机关集合, 不表达排布",
        ),
    },
    Mapping {
        gb_id: "header.doc_number",
        akn_target: "preface/p/docNumber + FRBRWork/FRBRnumber + FRBRWork/FRBRuri(工作层 IRI 含发文年份/序号)",
        kind: MappingKind::Direct,
        storage: "akn:doc/preface + akn:meta/identification",
        conflict: None,
    },
    Mapping {
        gb_id: "header.signers",
        akn_target: "workflow/step(@date,@by,@as) + preface/p/docProponent + references/TLCPerson,TLC Role",
        kind: MappingKind::NearFit,
        storage: "akn:meta/workflow + akn:doc/preface + akn:meta/references",
        conflict: Some(
            "AKN 无 docSignatory 元素; 签发人既是签名者(workflow 步骤), 又近似 docProponent(提议人), 语义不完全等价",
        ),
    },
    Mapping {
        gb_id: "header.red_line",
        akn_target: "无 AKN 语义; presentation/gbt:element[header.red_line]",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:elements",
        conflict: Some("红色分隔线是版头视觉分隔, AKN 无版头/分隔线概念"),
    },
    // ---------------- 主体 ----------------
    Mapping {
        gb_id: "body.title",
        akn_target: "preface/p/docTitle + FRBRWork/FRBRname; 版式在 presentation",
        kind: MappingKind::Direct,
        storage: "akn:doc/preface + akn:meta/identification",
        conflict: None,
    },
    Mapping {
        gb_id: "body.addressee",
        akn_target: "preface/p[@gbt:role='addressee'](纯文本); 无 AKN 元素",
        kind: MappingKind::Extension,
        storage: "akn:doc/preface",
        conflict: Some(
            "AKN 3.1 没有主送机关/收文者元素(docAddressee 不存在); 只能作为 preface 普通段落 + 扩展属性, 属重点待讨论项",
        ),
    },
    Mapping {
        gb_id: "body.text",
        akn_target: "mainBody/p(自然段)",
        kind: MappingKind::Direct,
        storage: "akn:doc/mainBody",
        conflict: None,
    },
    Mapping {
        gb_id: "body.hierarchy",
        akn_target: "mainBody/hcontainer[@name='level-1'..'level-4'](num,heading,intro,content,wrapUp) 嵌套",
        kind: MappingKind::Direct,
        storage: "akn:doc/mainBody",
        conflict: None,
    },
    Mapping {
        gb_id: "body.attachment_note",
        akn_target: "conclusions/p[@gbt:role='attachmentNote'] + attachments/attachment 清单",
        kind: MappingKind::Extension,
        storage: "akn:doc/conclusions",
        conflict: Some(
            "AKN 无“附件说明”元素; 附件说明是正文对附件的引用清单, 用 conclusions 段落 + 扩展属性表达, 附件实体在 attachments",
        ),
    },
    Mapping {
        gb_id: "body.signature_with_seal",
        akn_target: "conclusions/p/signature + FRBRauthor; 印章图形/位置在扩展",
        kind: MappingKind::NearFit,
        storage: "akn:doc/conclusions + akn:meta/identification",
        conflict: Some(
            "AKN 的 signature 是文本签名标识, 不表达红色印章图形、下压位置与“不得空白印章”等版式要求; 印章是图形资源",
        ),
    },
    Mapping {
        gb_id: "body.signature_without_seal",
        akn_target: "conclusions/p/signature + FRBRauthor",
        kind: MappingKind::NearFit,
        storage: "akn:doc/conclusions + akn:meta/identification",
        conflict: Some("署名与日期的相对位置(右空二字/首字右移二字)是版式, AKN 不表达"),
    },
    Mapping {
        gb_id: "body.signature_seal",
        akn_target: "conclusions/p/signature + workflow(签发人) + 签名章占位扩展",
        kind: MappingKind::NearFit,
        storage: "akn:doc/conclusions + akn:meta/workflow",
        conflict: Some(
            "签名章是图形资源; 签发人职务、签名章、成文日期的上下居中关系属版式; AKN 只能表达签发人身份与日期",
        ),
    },
    Mapping {
        gb_id: "body.date_digits",
        akn_target: "conclusions/p/date(@date=ISO) + FRBRdate; “年月日不编虚位”是呈现规则, 由渲染层执行",
        kind: MappingKind::Direct,
        storage: "akn:doc/conclusions + akn:meta/identification",
        conflict: None,
    },
    Mapping {
        gb_id: "body.special_spacing",
        akn_target: "无 AKN 语义; 由阶段 2 排版算法执行, 参数在 presentation",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:elements",
        conflict: Some("“调整行距、字距”是排版算法, AKN 无法表达"),
    },
    Mapping {
        gb_id: "body.annotation",
        akn_target: "meta/notes/note(@marker,@placement) + conclusions/p[@gbt:role='annotation']",
        kind: MappingKind::NearFit,
        storage: "akn:meta/notes + akn:doc/conclusions",
        conflict: Some(
            "AKN note 的语义是脚注/尾注/页边注; 公文附注是成文日期后的说明(如“此件公开发布”), 语义接近但不完全一致",
        ),
    },
    Mapping {
        gb_id: "body.attachment",
        akn_target: "attachments/attachment/doc(独立 FRBR 链)",
        kind: MappingKind::Direct,
        storage: "akn:doc/attachments",
        conflict: None,
    },
    Mapping {
        gb_id: "body.attachment_header",
        akn_target: "attachment/doc/preface/p[@gbt:role='attachmentLabel'|'attachmentTitle'] + docTitle",
        kind: MappingKind::Extension,
        storage: "akn:doc/attachments",
        conflict: Some(
            "“附件”二字与顺序号、附件标题在版心第三行是版式; AKN 用嵌套 doc 的 preface 承载说明文字",
        ),
    },
    Mapping {
        gb_id: "body.attachment_standalone",
        akn_target: "attachment/doc/preface/p[@gbt:role='attachmentStandaloneLabel'] + gbt:attachment-standalone",
        kind: MappingKind::Extension,
        storage: "akn:doc/attachments",
        conflict: Some("AKN 无“附件与正文分开装订时以发文字号标注”的概念"),
    },
    // ---------------- 版记 ----------------
    Mapping {
        gb_id: "record.separator_lines",
        akn_target: "无 AKN 语义; presentation/gbt:record-lines",
        kind: MappingKind::NoFit,
        storage: "akn:presentation",
        conflict: Some("版记分隔线(粗 0.35/细 0.25 mm)是纯版式"),
    },
    Mapping {
        gb_id: "record.cc",
        akn_target: "proprietary/gbt:gbt9704/gbt:record/gbt:cc(备选: conclusions/container[@name='版记'])",
        kind: MappingKind::Extension,
        storage: "akn:meta/proprietary",
        conflict: Some(
            "AKN 3.1 无抄送机关元素; 与主送机关同属“收文者”缺口, 只能扩展, 属重点待讨论项; conclusions 里的 container@name='版记' 是合法原生容器, 但 doc 在 attachments 之后没有容器位置, 故统一放 proprietary",
        ),
    },
    Mapping {
        gb_id: "record.addressee_moved",
        akn_target: "proprietary/gbt:gbt9704/gbt:record/gbt:addresseeMoved",
        kind: MappingKind::Extension,
        storage: "akn:meta/proprietary",
        conflict: Some("主送机关移入版记是版式位置变化, AKN 只记录“哪些机关在版记中”"),
    },
    Mapping {
        gb_id: "record.printing",
        akn_target: "meta/publication(@name,@date) + proprietary/gbt:record/gbt:printing",
        kind: MappingKind::NearFit,
        storage: "akn:meta/publication + akn:meta/proprietary",
        conflict: Some(
            "publication 的语义是官方公布事件(公报/公告), 印发是机关内部印制发行; 语义偏移, 显示文本另存扩展",
        ),
    },
    Mapping {
        gb_id: "record.other_elements",
        akn_target: "proprietary/gbt:record + presentation/gbt:record-lines",
        kind: MappingKind::Extension,
        storage: "akn:meta/proprietary + akn:presentation",
        conflict: Some("版记其他要素无 AKN 语义, 只能结构化扩展"),
    },
    // ---------------- 页码 ----------------
    Mapping {
        gb_id: "table.landscape",
        akn_target: "akn:table + presentation(横排/表头方向)",
        kind: MappingKind::NearFit,
        storage: "akn:doc/mainBody/table + akn:presentation",
        conflict: Some("表格结构可原生表达; 横排纸张、单双页表头在订口/切口属版式, 需渲染层处理"),
    },
    Mapping {
        gb_id: "page.number",
        akn_target: "无 AKN 语义; presentation/gbt:page-number",
        kind: MappingKind::NoFit,
        storage: "akn:presentation",
        conflict: Some(
            "页码是 manifestation 的呈现属性, AKN 无页码元素; eop/@number 只能记录原文分页锚点, 单双页位置/一字线/空白页规则属版式",
        ),
    },
    // ---------------- 内容规则 ----------------
    Mapping {
        gb_id: "content.units",
        akn_target: "无 AKN 语义; 由内容校验/排版层执行, 规则记录在 presentation",
        kind: MappingKind::NoFit,
        storage: "akn:presentation",
        conflict: Some(
            "计量单位/标点/数字用法是文字内容规范, 不是文档结构; AKN 不表达, 应作为校验规则",
        ),
    },
    // ---------------- 特定格式 ----------------
    Mapping {
        gb_id: "special.letter",
        akn_target: "FRBRWork/FRBRsubtype(信函) + doc + presentation/gbt:format[name=letter]",
        kind: MappingKind::NearFit,
        storage: "akn:meta/identification + akn:presentation",
        conflict: Some(
            "AKN 无信函根元素/版式; 用通用 doc + FRBRsubtype 表达文种, 版式参数(170 mm 双线/上 30 mm/下 20 mm)进扩展",
        ),
    },
    Mapping {
        gb_id: "special.order",
        akn_target: "FRBRWork/FRBRsubtype(命令) + doc + presentation/gbt:format[name=order]",
        kind: MappingKind::NearFit,
        storage: "akn:meta/identification + akn:presentation",
        conflict: Some(
            "AKN 的 act/bill 等不覆盖中国行政命令(令); 使用通用 doc + FRBRsubtype + 版式扩展",
        ),
    },
    Mapping {
        gb_id: "special.minutes",
        akn_target: "FRBRWork/FRBRsubtype(纪要) + conclusions/p[@gbt:role='attendance'] + presentation",
        kind: MappingKind::NearFit,
        storage: "akn:meta/identification + akn:doc/conclusions + akn:presentation",
        conflict: Some(
            "AKN 的 debate/debateReport 面向议会辩论记录, 与中文“纪要”不同; 出席/请假/列席名单无原生元素, 用 conclusions 段落 + 扩展属性",
        ),
    },
    // ---------------- 式样 ----------------
    Mapping {
        gb_id: "sample.figures",
        akn_target: "无 AKN 语义; docs/figures + presentation/gbt:element[sample.figures]",
        kind: MappingKind::NoFit,
        storage: "akn:presentation/gbt:elements",
        conflict: Some("式样图是标准的说明性附图, 不属于任何具体公文实例"),
    },
];

/// 标准条款粒度覆盖: `docs/gbt9704-2012-elements.json` 中 77 个细分要素 id → 本表映射单元.
///
/// 本表把 GB/T 9704 的"最小条款项"(如 page.margin_top、body.hierarchy_level_2、
/// letterhead.double_line_top)全部挂到上层映射单元上, 保证"标准里的每一项都有去向",
/// 同时不把映射表膨胀成 77 条重复描述.
pub const GRANULAR_COVERAGE: &[(&str, &str)] = &[
    // 纸张/版面/术语
    ("page.paper", "paper.a4"),
    ("page.type_area", "layout.margins"),
    ("page.margin_top", "layout.margins"),
    ("page.margin_left", "layout.margins"),
    ("page.margin_bottom", "layout.margins"),
    ("page.margin_right", "layout.margins"),
    ("page.line_grid", "layout.grid"),
    ("page.char_grid", "layout.grid"),
    ("page.font_default", "layout.default_font"),
    ("page.text_color", "layout.text_color"),
    ("unit.char", "layout.units"),
    ("unit.line", "layout.units"),
    // 印制装订
    ("print.plate", "printing.plate"),
    ("print.duplex", "printing.ink"),
    ("print.registration", "printing.ink"),
    ("print.ink_black", "printing.ink"),
    ("print.ink_red", "printing.ink"),
    ("bind.side", "printing.binding"),
    ("bind.pagination_tolerance", "printing.binding"),
    ("bind.trim_tolerance", "printing.binding"),
    ("bind.staple_position", "printing.binding"),
    ("bind.staple_riding", "printing.binding"),
    ("bind.staple_side", "printing.binding"),
    ("bind.wrapped", "printing.binding"),
    // 版头
    ("header.copy_number", "header.copy_number"),
    ("header.secrecy", "header.secrecy"),
    ("header.urgency", "header.urgency"),
    ("header.authority_mark", "header.issuer_mark"),
    ("header.doc_number", "header.doc_number"),
    ("header.signatory", "header.signers"),
    ("header.separator_line", "header.red_line"),
    // 主体
    ("body.title", "body.title"),
    ("body.main_recipient", "body.addressee"),
    ("body.text", "body.text"),
    ("body.hierarchy_level_1", "body.hierarchy"),
    ("body.hierarchy_level_2", "body.hierarchy"),
    ("body.hierarchy_level_3", "body.hierarchy"),
    ("body.hierarchy_level_4", "body.hierarchy"),
    ("body.attachment_note", "body.attachment_note"),
    ("body.attachment_note_item", "body.attachment_note"),
    ("body.issuing_signature", "body.signature_with_seal"),
    ("body.issue_date", "body.date_digits"),
    ("body.seal", "body.signature_with_seal"),
    ("body.signatory_title", "body.signature_seal"),
    ("body.signatory_seal", "body.signature_seal"),
    ("body.annotation", "body.annotation"),
    ("body.attachment", "body.attachment"),
    ("body.attachment_label", "body.attachment_header"),
    ("body.attachment_title", "body.attachment_header"),
    // 版记
    ("record.separator_thick", "record.separator_lines"),
    ("record.separator_thin", "record.separator_lines"),
    ("record.cc", "record.cc"),
    ("record.main_recipient", "record.addressee_moved"),
    ("record.printing_authority", "record.printing"),
    ("record.printing_date", "record.printing"),
    // 页码
    ("page.number", "page.number"),
    ("page.number_dash", "page.number"),
    // 横排表格
    ("table.landscape.header_side", "table.landscape"),
    ("table.landscape.page_number", "table.landscape"),
    // 信函格式
    ("letterhead.authority_mark", "special.letter"),
    ("letterhead.double_line_top", "special.letter"),
    ("letterhead.double_line_bottom", "special.letter"),
    ("letterhead.header_elements", "special.letter"),
    ("letterhead.doc_number", "special.letter"),
    ("letterhead.title", "special.letter"),
    ("letterhead.bottom_line_text", "special.letter"),
    ("letterhead.page_number", "special.letter"),
    ("letterhead.record", "special.letter"),
    // 命令(令)格式
    ("order.authority_mark", "special.order"),
    ("order.order_number", "special.order"),
    ("order.body", "special.order"),
    ("order.signatory", "special.order"),
    // 纪要格式
    ("minutes.mark", "special.minutes"),
    ("minutes.attendance", "special.minutes"),
    ("minutes.leave_absent", "special.minutes"),
    ("minutes.flexible", "special.minutes"),
    // 规范性引用文件
    ("ref.standards", "content.units"),
];

/// 按 gb_id 查映射.
pub fn find(gb_id: &str) -> Option<&'static Mapping> {
    MAPPINGS.iter().find(|mapping| mapping.gb_id == gb_id)
}

/// 自检: 映射表与要素注册表必须一一对应, 且分类合理.
pub fn check_consistency() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    for element in elements::ELEMENTS {
        if find(element.id).is_none() {
            errors.push(format!("要素 {} 没有映射", element.id));
        }
    }
    for mapping in MAPPINGS {
        if elements::find(mapping.gb_id).is_none() {
            errors.push(format!("映射 {} 指向不存在的要素", mapping.gb_id));
        }
        if mapping.kind == MappingKind::Direct && mapping.conflict.is_some() {
            errors.push(format!("{} 标记为直接对应却给出了冲突说明", mapping.gb_id));
        }
        if matching(mapping.kind) && mapping.conflict.is_none() {
            errors.push(format!("{} 不是直接对应, 必须写明冲突/偏移", mapping.gb_id));
        }
    }
    if MAPPINGS.len() != elements::ELEMENTS.len() {
        errors.push(format!(
            "映射数 {} 与要素数 {} 不一致",
            MAPPINGS.len(),
            elements::ELEMENTS.len()
        ));
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn matching(kind: MappingKind) -> bool {
    kind != MappingKind::Direct
}

fn count(kind: MappingKind) -> usize {
    MAPPINGS
        .iter()
        .filter(|mapping| mapping.kind == kind)
        .count()
}

/// 生成 Markdown 版映射文档(`docs/mapping-gbt9704-akn.md`).
pub fn render_markdown() -> String {
    let mut out = String::new();
    out.push_str("# GB/T 9704-2012 → Akoma Ntoso 3.1 全量映射表\n\n");
    out.push_str(
        "> 本文件由 `cargo run -p gbt9704 -- mapping` 生成; 修改请改 `src/mapping.rs`。\n\n",
    );

    out.push_str("## 1. 目标标准\n\n");
    out.push_str("- 公文格式: **GB/T 9704-2012《党政机关公文格式》** (2012-06-29 发布, 2012-07-01 实施, 代替 GB/T 9704-1999)\n");
    out.push_str("- Akoma Ntoso: **Akoma Ntoso Version 2.0, Part 2: Specifications of AKN 3.1** (OASIS LegalDocML, OASIS Standard, 2026-10-02 生效)\n");
    out.push_str("- 命名空间: 默认口径 A `http://docs.oasis-open.org/legaldocml/ns/akn/3.0` + 根元素 `version=\"3.1\"` (官方 XSD 可校验); 可选口径 B `.../ns/akn/3.1` (OASIS v2.0 命名空间文档声明, 官方未随发对应 XSD, 需本地 adapted schema)\n");
    out.push_str("- 扩展命名空间: `urn:gbt9704:ext:1.0`, 只出现在 `akn:proprietary` / `akn:presentation` 内部\n\n");

    out.push_str("## 2. 映射类别\n\n");
    out.push_str("| 类别 | 英文 id | 含义 | 数量 |\n|---|---|---|---|\n");
    for kind in [
        MappingKind::Direct,
        MappingKind::NearFit,
        MappingKind::Extension,
        MappingKind::NoFit,
    ] {
        out.push_str(&format!(
            "| {} | `{}` | {} | {} |\n",
            kind.zh(),
            kind.id(),
            kind.description(),
            count(kind)
        ));
    }
    out.push_str(&format!(
        "\n合计: **{}** 项标准要素, 全部有明确去向; 其中存在语义偏移/无原生对应(**需讨论**)的有 **{}** 项。\n\n",
        MAPPINGS.len(),
        count(MappingKind::NearFit) + count(MappingKind::Extension) + count(MappingKind::NoFit)
    ));

    out.push_str("## 3. 全量映射表\n\n");
    for part in [
        Part::Paper,
        Part::Layout,
        Part::Printing,
        Part::Header,
        Part::Body,
        Part::Record,
        Part::PageNumber,
        Part::Table,
        Part::ContentRules,
        Part::Special,
        Part::Sample,
    ] {
        let part_elements: Vec<&ElementSpec> = elements::ELEMENTS
            .iter()
            .filter(|element| element.part == part)
            .collect();
        if part_elements.is_empty() {
            continue;
        }
        out.push_str(&format!("### 3.{} {}\n\n", part_index(part), part.zh()));
        out.push_str("| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |\n");
        out.push_str("|---|---|---|---|---|\n");
        for element in part_elements {
            let mapping = find(element.id).expect("映射自检应保证存在");
            out.push_str(&format!(
                "| {} (`{}`) | {} | {} | {} | {} |\n",
                element.name,
                element.id,
                element.clause,
                mapping.kind.zh(),
                escape_md(mapping.akn_target),
                mapping
                    .conflict
                    .map(escape_md)
                    .unwrap_or_else(|| "无".to_string())
            ));
        }
        out.push('\n');
    }

    out.push_str("## 4. 冲突与待讨论清单\n\n");
    out.push_str("### 4.1 语义近似(AKN 有相近元素, 但语义偏移)\n\n");
    for mapping in MAPPINGS.iter().filter(|m| m.kind == MappingKind::NearFit) {
        let element = elements::find(mapping.gb_id).expect("自检保证");
        out.push_str(&format!(
            "- **{}** ({}) → {}; 冲突: {}\n",
            element.name,
            element.clause,
            escape_md(mapping.akn_target),
            escape_md(mapping.conflict.unwrap_or(""))
        ));
    }
    out.push_str("\n### 4.2 AKN 无原生元素, 用扩展承载\n\n");
    for mapping in MAPPINGS.iter().filter(|m| m.kind == MappingKind::Extension) {
        let element = elements::find(mapping.gb_id).expect("自检保证");
        out.push_str(&format!(
            "- **{}** ({}) → {}; 冲突: {}\n",
            element.name,
            element.clause,
            escape_md(mapping.akn_target),
            escape_md(mapping.conflict.unwrap_or(""))
        ));
    }
    out.push_str("\n### 4.3 纯版式/印制/内容规则(不进 AKN 语义)\n\n");
    for mapping in MAPPINGS.iter().filter(|m| m.kind == MappingKind::NoFit) {
        let element = elements::find(mapping.gb_id).expect("自检保证");
        out.push_str(&format!(
            "- **{}** ({}) → {}; 冲突: {}\n",
            element.name,
            element.clause,
            escape_md(mapping.akn_target),
            escape_md(mapping.conflict.unwrap_or(""))
        ));
    }

    out.push_str("\n## 5. 标准条款粒度覆盖(77 项)\n\n");
    out.push_str("`docs/gbt9704-2012-elements.json` 把标准拆成 77 个细分要素; 下表说明每一项落到哪个映射单元, 因此\n");
    out.push_str("“标准里的每一项”都有明确去向(细分项继承所属映射单元的类别与扩展方式)。\n\n");
    out.push_str("| 细分要素 id | 名称/条款 | 归属映射单元 | 类别 |\n|---|---|---|---|\n");
    for (std_id, parent) in GRANULAR_COVERAGE {
        let parent_mapping = find(parent).expect("粒度覆盖的父映射必须存在");
        let name = granular_name(std_id);
        out.push_str(&format!(
            "| `{}` | {} | `{}` | {} |\n",
            std_id,
            name,
            parent,
            parent_mapping.kind.zh()
        ));
    }
    out.push('\n');

    out.push_str("\n## 6. 设计约定(实现层)\n\n");
    out.push_str("1. **AKN 是唯一中间状态**: 语义内容写 AKN 原生元素, 版式写 `presentation`, AKN 没有的元数据写 `proprietary`; 不再维护第二份私有格式。\n");
    out.push_str("2. **FRBR**: 每份公文生成 Work/Expression/Manifestation 三层 IRI; 发文字号进 `FRBRnumber`, 标题进 `FRBRname`, 文种进 `FRBRsubtype`, 机关进 `FRBRauthor`。\n");
    out.push_str("3. **扩展块**: `akn:proprietary` 承载版记(抄送/印发/移入版记的主送机关)等非呈现元数据; `akn:presentation` 承载全部版面/字体/位置/页码/要素规格(含标准每一个要素)。\n");
    out.push_str("4. **可见内容与机器元数据并存**: 标题/发文字号/成文日期等同时出现在 `preface`/`conclusions` 的可读内容与 `meta` 的 FRBR/classification 中, 这是 AKN 的既定用法(内容用于阅读, meta 用于机器处理)。\n");
    out.push_str("5. **eId 全局唯一**: 包括嵌套附件 `doc` 内的元素; 由生成器统一编号。\n");
    out.push_str("6. **未表达即未定义**: 阶段 2 渲染只允许使用 AKN 中已有的信息; 若渲染需要新参数, 必须先扩展 `presentation` 并更新本表。\n\n");

    out.push_str("## 7. 自检结果\n\n");
    match check_consistency() {
        Ok(()) => out.push_str(&format!(
            "- [x] {} 项标准要素全部有映射, 无遗漏、无悬空引用\n",
            MAPPINGS.len()
        )),
        Err(errors) => {
            out.push_str("- [ ] 映射自检失败:\n");
            for error in errors {
                out.push_str(&format!("  - {error}\n"));
            }
        }
    }
    out.push_str("- [x] 每个非“直接对应”项都写明了冲突/偏移\n");
    out
}

/// 细分要素的中文名/条款号(以 inventory 为准的简表; 完整字段见 docs JSON).
fn granular_name(std_id: &str) -> &'static str {
    match std_id {
        "page.paper" => "公文用纸幅面(5.1)",
        "page.type_area" => "版心尺寸(5.2.1)",
        "page.margin_top" => "天头 37±1 mm(5.2.1)",
        "page.margin_left" => "订口 28±1 mm(5.2.1)",
        "page.margin_bottom" => "地脚 35 mm(5.2.1 推导)",
        "page.margin_right" => "切口 26 mm(5.2.1 推导)",
        "page.line_grid" => "22 行网格(5.2.3)",
        "page.char_grid" => "28 字网格(5.2.3)",
        "page.font_default" => "默认 3 号仿宋(5.2.2)",
        "page.text_color" => "文字黑色(5.2.4)",
        "unit.char" => "字(3.1)",
        "unit.line" => "行(3.2)",
        "print.plate" => "制版(6.1)",
        "print.duplex" => "双面印刷(6.2)",
        "print.registration" => "页码套正(6.2)",
        "print.ink_black" => "黑墨 BL100%(6.2)",
        "print.ink_red" => "红墨 Y80%+M80%(6.2)",
        "bind.side" => "左侧装订(6.3)",
        "bind.pagination_tolerance" => "页码误差 4 mm(6.3)",
        "bind.trim_tolerance" => "成品误差±2 mm(6.3)",
        "bind.staple_position" => "钉位距上下边缘 70 mm(6.3)",
        "bind.staple_riding" => "骑马订(6.3)",
        "bind.staple_side" => "平订(6.3)",
        "bind.wrapped" => "包本装订(6.3)",
        "header.copy_number" => "份号(7.2.1)",
        "header.secrecy" => "密级和保密期限(7.2.2)",
        "header.urgency" => "紧急程度(7.2.3)",
        "header.authority_mark" => "发文机关标志(7.2.4)",
        "header.doc_number" => "发文字号(7.2.5)",
        "header.signatory" => "签发人(7.2.6)",
        "header.separator_line" => "版头红色分隔线(7.2.7)",
        "body.title" => "标题(7.3.1)",
        "body.main_recipient" => "主送机关(7.3.2)",
        "body.text" => "正文(7.3.3)",
        "body.hierarchy_level_1" => "一级序数“一、”(7.3.3)",
        "body.hierarchy_level_2" => "二级序数“(一)”(7.3.3)",
        "body.hierarchy_level_3" => "三级序数“1.”(7.3.3)",
        "body.hierarchy_level_4" => "四级序数“(1)”(7.3.3)",
        "body.attachment_note" => "附件说明(7.3.4)",
        "body.attachment_note_item" => "附件说明条目(7.3.4)",
        "body.issuing_signature" => "发文机关署名(7.3.5.1/2)",
        "body.issue_date" => "成文日期(7.3.5.4)",
        "body.seal" => "印章(7.3.5.1)",
        "body.signatory_title" => "签发人职务(7.3.5.3)",
        "body.signatory_seal" => "签发人签名章(7.3.5.3)",
        "body.annotation" => "附注(7.3.6)",
        "body.attachment" => "附件(7.3.7)",
        "body.attachment_label" => "“附件”标注(7.3.7)",
        "body.attachment_title" => "附件标题(7.3.7)",
        "record.separator_thick" => "版记粗分隔线(7.4.1)",
        "record.separator_thin" => "版记细分隔线(7.4.1)",
        "record.cc" => "抄送机关(7.4.2)",
        "record.main_recipient" => "主送机关移入版记(7.4.2)",
        "record.printing_authority" => "印发机关(7.4.3)",
        "record.printing_date" => "印发日期(7.4.3)",
        "page.number" => "页码(7.5)",
        "page.number_dash" => "页码一字线(7.5)",
        "table.landscape.header_side" => "横排表格表头方向(8)",
        "table.landscape.page_number" => "横排表格页码位置(8)",
        "letterhead.authority_mark" => "信函发文机关标志(10.1)",
        "letterhead.double_line_top" => "信函上红色双线(10.1)",
        "letterhead.double_line_bottom" => "信函下红色双线(10.1)",
        "letterhead.header_elements" => "信函份号/密级/紧急程度(10.1)",
        "letterhead.doc_number" => "信函发文字号(10.1)",
        "letterhead.title" => "信函标题(10.1)",
        "letterhead.bottom_line_text" => "下双线上文字(10.1)",
        "letterhead.page_number" => "信函首页不编页码(10.1)",
        "letterhead.record" => "信函版记(10.1)",
        "order.authority_mark" => "命令发文机关标志(10.2)",
        "order.order_number" => "令号(10.2)",
        "order.body" => "命令正文(10.2)",
        "order.signatory" => "命令签发人/签名章/日期(10.2→7.3.5.3)",
        "minutes.mark" => "纪要标志(10.3)",
        "minutes.attendance" => "出席人员名单(10.3)",
        "minutes.leave_absent" => "请假/列席人员名单(10.3)",
        "minutes.flexible" => "纪要格式可自定(10.3)",
        "ref.standards" => "规范性引用文件(2)",
        _ => "未知细分要素",
    }
}

fn part_index(part: Part) -> usize {
    match part {
        Part::Paper => 1,
        Part::Layout => 2,
        Part::Printing => 3,
        Part::Header => 4,
        Part::Body => 5,
        Part::Record => 6,
        Part::PageNumber => 7,
        Part::Table => 8,
        Part::ContentRules => 9,
        Part::Special => 10,
        Part::Sample => 11,
    }
}

fn escape_md(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

/// 生成机器可读 JSON.
pub fn render_json() -> String {
    let mut items = Vec::new();
    for mapping in MAPPINGS {
        let element = elements::find(mapping.gb_id).expect("映射自检应保证存在");
        items.push(Value::object(vec![
            ("gb_id", Value::string(element.id)),
            ("name", Value::string(element.name)),
            ("clause", Value::string(element.clause)),
            ("part", Value::string(element.part.zh())),
            ("kind", Value::string(mapping.kind.id())),
            ("akn_target", Value::string(mapping.akn_target)),
            ("storage", Value::string(mapping.storage)),
            (
                "conflict",
                mapping.conflict.map(Value::string).unwrap_or(Value::Null),
            ),
        ]));
    }
    let value = Value::object(vec![
        (
            "standard",
            Value::object(vec![
                ("id", Value::string("GB/T 9704-2012")),
                ("name", Value::string("党政机关公文格式")),
            ]),
        ),
        (
            "akn",
            Value::object(vec![
                ("version", Value::string(crate::akn::AKN_VERSION)),
                ("namespace", Value::string(crate::akn::NS_AKN)),
                ("extension_namespace", Value::string(crate::akn::NS_GBT)),
            ]),
        ),
        (
            "summary",
            Value::object(vec![
                ("total", Value::Number(MAPPINGS.len() as f64)),
                ("direct", Value::Number(count(MappingKind::Direct) as f64)),
                (
                    "near_fit",
                    Value::Number(count(MappingKind::NearFit) as f64),
                ),
                (
                    "extension",
                    Value::Number(count(MappingKind::Extension) as f64),
                ),
                ("no_fit", Value::Number(count(MappingKind::NoFit) as f64)),
                ("granular", Value::Number(GRANULAR_COVERAGE.len() as f64)),
            ]),
        ),
        ("mappings", Value::Array(items)),
        (
            "granular_coverage",
            Value::Array(
                GRANULAR_COVERAGE
                    .iter()
                    .map(|(std_id, parent)| {
                        Value::object(vec![
                            ("std_id", Value::string(*std_id)),
                            ("parent_mapping", Value::string(*parent)),
                            ("name", Value::string(granular_name(std_id))),
                        ])
                    })
                    .collect(),
            ),
        ),
    ]);
    json::to_pretty(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mapping_table_is_consistent() {
        check_consistency().unwrap();
    }

    #[test]
    fn markdown_covers_every_element() {
        let markdown = render_markdown();
        for element in elements::ELEMENTS {
            assert!(
                markdown.contains(element.id),
                "Markdown 缺少要素 {}",
                element.id
            );
        }
        assert!(markdown.contains("冲突与待讨论清单"));
    }

    #[test]
    fn json_is_valid() {
        let text = render_json();
        let value = json::parse(&text).expect("render_json 必须产生合法 JSON");
        assert_eq!(
            value.get("summary").unwrap().get("total").unwrap().as_f64(),
            Some(elements::ELEMENTS.len() as f64)
        );
    }
}

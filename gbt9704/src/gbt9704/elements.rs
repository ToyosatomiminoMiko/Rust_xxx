//! GB/T 9704-2012 格式要素注册表 (机器可读的"标准项").
//!
//! 每条记录对应标准中的一个可编排项, 覆盖:
//! 纸张/印制(第 4、6 章), 版面(第 5 章), 版头/主体/版记/页码(第 7 章),
//! 横排表格(第 8 章), 计量单位与标点数字(第 9 章), 特定格式(第 10 章), 式样(第 11 章).
//!
//! 本表是 `mapping` 模块与 `docs/gbt9704-2012-elements.json` 的代码侧事实来源;
//! 修改标准相关数值时只改这里 + `geometry`/`fonts`.
//!
//! 说明: "required" 只表示该要素在对应情形下是否必须出现, 不表示每份公文都必须写.
//! 标准大量使用"如无特殊说明/一般/如需/可以", 这里用 `Requirement::Conditional` 保留其弹性.

use super::fonts::FontSlot;
use super::geometry::{BLACK, Color, RED};

/// 格式要素所属部分.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Part {
    /// 公文用纸技术指标(第 4 章).
    Paper,
    /// 幅面与版面(第 5 章).
    Layout,
    /// 印制装订(第 6 章).
    Printing,
    /// 版头(7.2).
    Header,
    /// 主体(7.3).
    Body,
    /// 版记(7.4).
    Record,
    /// 页码(7.5).
    PageNumber,
    /// 特定格式(第 10 章).
    Special,
    /// 横排表格(第 8 章).
    Table,
    /// 计量单位/标点/数字(第 9 章).
    ContentRules,
    /// 式样(第 11 章), 仅作参考, 不直接参与排版.
    Sample,
}

impl Part {
    pub fn zh(self) -> &'static str {
        match self {
            Part::Paper => "用纸",
            Part::Layout => "版面",
            Part::Printing => "印制装订",
            Part::Header => "版头",
            Part::Body => "主体",
            Part::Record => "版记",
            Part::PageNumber => "页码",
            Part::Special => "特定格式",
            Part::Table => "横排表格",
            Part::ContentRules => "计量单位与标点数字",
            Part::Sample => "式样",
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Part::Paper => "paper",
            Part::Layout => "layout",
            Part::Printing => "printing",
            Part::Header => "header",
            Part::Body => "body",
            Part::Record => "record",
            Part::PageNumber => "page_number",
            Part::Special => "special",
            Part::Table => "table",
            Part::ContentRules => "content_rules",
            Part::Sample => "sample",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id {
            "paper" => Part::Paper,
            "layout" => Part::Layout,
            "printing" => Part::Printing,
            "header" => Part::Header,
            "body" => Part::Body,
            "record" => Part::Record,
            "page_number" => Part::PageNumber,
            "special" => Part::Special,
            "table" => Part::Table,
            "content_rules" => Part::ContentRules,
            "sample" => Part::Sample,
            _ => return None,
        })
    }
}

/// 要素出现要求.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Requirement {
    /// 必须出现.
    Required,
    /// 由发文机关按需决定是否标注.
    Optional,
    /// 满足条件时必须/应当按规定编排(标准常用"如需/一般/如无特殊说明").
    Conditional,
}

impl Requirement {
    /// 稳定 id, 用于 XML/JSON.
    pub fn id(self) -> &'static str {
        match self {
            Requirement::Required => "required",
            Requirement::Optional => "optional",
            Requirement::Conditional => "conditional",
        }
    }

    pub fn zh(self) -> &'static str {
        match self {
            Requirement::Required => "必须",
            Requirement::Optional => "可选",
            Requirement::Conditional => "条件",
        }
    }
}

/// 编排对齐方式(仅记录标准的主要要求, 细节见 position/rules).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment {
    Left,
    Center,
    Right,
    Justify,
    /// 顶格(贴版心左上角).
    TopLeft,
    /// 无文字内容(如分隔线/纸张), 对齐不适用.
    NotApplicable,
}

impl Alignment {
    /// 稳定 id, 用于 XML/JSON.
    pub fn id(self) -> &'static str {
        match self {
            Alignment::Left => "left",
            Alignment::Center => "center",
            Alignment::Right => "right",
            Alignment::Justify => "justify",
            Alignment::TopLeft => "top_left",
            Alignment::NotApplicable => "not_applicable",
        }
    }

    pub fn zh(self) -> &'static str {
        match self {
            Alignment::Left => "居左",
            Alignment::Center => "居中",
            Alignment::Right => "居右",
            Alignment::Justify => "两端对齐",
            Alignment::TopLeft => "顶格",
            Alignment::NotApplicable => "不适用",
        }
    }
}

/// 一个格式要素的规格.
#[derive(Debug, Clone, Copy)]
pub struct ElementSpec {
    /// 稳定 id, 形如 `header.copy_number`.
    pub id: &'static str,
    /// 标准中的名称.
    pub name: &'static str,
    pub part: Part,
    pub requirement: Requirement,
    /// 字体槽位; 无文字内容的要素为 None.
    pub font: Option<FontSlot>,
    /// 字号; 标准未规定或非文字要素为 None.
    pub font_size_pt: Option<f64>,
    pub color: Color,
    pub align: Alignment,
    /// 位置要求(相对关系, 便于渲染器计算).
    pub position: &'static str,
    /// 间距/空行要求.
    pub spacing: &'static str,
    /// 其余强制或推荐规则.
    pub rules: &'static str,
    /// 标准条款号.
    pub clause: &'static str,
}

/// 全部要素规格.
pub const ELEMENTS: &[ElementSpec] = &[
    // ------------------------------------------------------------------
    // 第 4 章 公文用纸主要技术指标
    // ------------------------------------------------------------------
    ElementSpec {
        id: "paper.technical",
        name: "公文用纸技术指标",
        part: Part::Paper,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "不涉及",
        spacing: "不涉及",
        rules: "定量 60 g/m²~80 g/m² 的胶版印刷纸或复印纸; 白度 80%~90%; 横向耐折度≥15 次; 不透明度≥85%; pH 7.5~9.5",
        clause: "4",
    },
    // ------------------------------------------------------------------
    // 第 5 章 幅面尺寸及版面要求
    // ------------------------------------------------------------------
    ElementSpec {
        id: "paper.a4",
        name: "幅面尺寸(A4)",
        part: Part::Layout,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "成品幅面 210 mm×297 mm",
        spacing: "不涉及",
        rules: "采用 GB/T 148 规定的 A4 型纸",
        clause: "5.1",
    },
    ElementSpec {
        id: "layout.margins",
        name: "页边与版心尺寸",
        part: Part::Layout,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "天头(上白边)37 mm±1 mm; 订口(左白边)28 mm±1 mm; 版心 156 mm×225 mm",
        spacing: "右侧白边 26 mm、下白边 35 mm 由幅面与版心推导",
        rules: "版心位置由天头与订口确定, 误差不超过 1 mm(6.1)",
        clause: "5.2.1",
    },
    ElementSpec {
        id: "layout.default_font",
        name: "字体和字号(默认)",
        part: Part::Layout,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "适用于版心内各要素, 除非该要素另有规定",
        spacing: "不涉及",
        rules: "如无特殊说明, 公文格式各要素一般用 3 号仿宋体字; 特定情况可以作适当调整",
        clause: "5.2.2",
    },
    ElementSpec {
        id: "layout.grid",
        name: "行数和字数",
        part: Part::Layout,
        requirement: Requirement::Conditional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "版心内 22 行 × 28 字, 撑满版心",
        spacing: "网格行距 225/22 mm≈28.99 pt; 网格字距 156/28 mm≈15.79 pt",
        rules: "一般每面排 22 行, 每行排 28 个字, 并撑满版心; 特定情况可以作适当调整",
        clause: "5.2.3",
    },
    ElementSpec {
        id: "layout.units",
        name: "字、行(术语定义)",
        part: Part::Layout,
        requirement: Requirement::Conditional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "不涉及",
        spacing: "1 字 = 3 号汉字宽度 = 16 pt = 5.6444 mm; 1 行 = 汉字高度 + 3 号汉字高度的 7/8 = 30 pt = 10.5833 mm",
        rules: "3.1/3.2 定义的横向/纵向距离单位; 与版面网格(字距 156/28 mm、行距 225/22 mm)是两套常量, 实现不得混用",
        clause: "3.1/3.2",
    },
    ElementSpec {
        id: "layout.text_color",
        name: "文字的颜色",
        part: Part::Layout,
        requirement: Requirement::Conditional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "不涉及",
        spacing: "不涉及",
        rules: "如无特殊说明, 公文中文字的颜色均为黑色",
        clause: "5.2.4",
    },
    // ------------------------------------------------------------------
    // 第 6 章 印制装订要求
    // ------------------------------------------------------------------
    ElementSpec {
        id: "printing.plate",
        name: "制版要求",
        part: Part::Printing,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "不涉及",
        spacing: "不涉及",
        rules: "版面干净无底灰, 字迹清楚无断划, 尺寸标准, 版心不斜, 误差不超过 1 mm",
        clause: "6.1",
    },
    ElementSpec {
        id: "printing.ink",
        name: "印刷要求",
        part: Part::Printing,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: RED,
        align: Alignment::NotApplicable,
        position: "不涉及",
        spacing: "不涉及",
        rules: "双面印刷; 页码套正, 两面误差不超过 2 mm; 黑色油墨达到色谱 BL100%, 红色油墨达到 Y80%、M80%",
        clause: "6.2",
    },
    ElementSpec {
        id: "printing.binding",
        name: "装订要求",
        part: Part::Printing,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "左侧装订; 骑马订或平订订位为两钉外订眼距版面上下边缘各 70 mm 处(±4 mm)",
        spacing: "两页页码之间误差不超过 4 mm; 成品尺寸允许误差±2 mm",
        rules: "不掉页, 四角成 90°, 无毛茬或缺损; 平订钉锯与书脊间距离 3 mm~5 mm; 包本装订封皮与书芯吻合、包紧、包平、不脱落",
        clause: "6.3",
    },
    // ------------------------------------------------------------------
    // 7.2 版头
    // ------------------------------------------------------------------
    ElementSpec {
        id: "header.copy_number",
        name: "份号",
        part: Part::Header,
        requirement: Requirement::Optional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::TopLeft,
        position: "版心左上角第一行, 顶格",
        spacing: "无额外空行",
        rules: "一般用 6 位 3 号阿拉伯数字标注",
        clause: "7.2.1",
    },
    ElementSpec {
        id: "header.secrecy",
        name: "密级和保密期限",
        part: Part::Header,
        requirement: Requirement::Optional,
        font: Some(FontSlot::Heiti),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::TopLeft,
        position: "版心左上角第二行, 顶格(无份号时上移)",
        spacing: "无额外空行",
        rules: "一般用 3 号黑体字; 保密期限中的数字用阿拉伯数字标注; 与份号、紧急程度按顺序自上而下分行排列",
        clause: "7.2.2",
    },
    ElementSpec {
        id: "header.urgency",
        name: "紧急程度",
        part: Part::Header,
        requirement: Requirement::Optional,
        font: Some(FontSlot::Heiti),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::TopLeft,
        position: "版心左上角, 按份号、密级和保密期限、紧急程度的顺序自上而下分行顶格排列",
        spacing: "无额外空行",
        rules: "一般用 3 号黑体字",
        clause: "7.2.3",
    },
    ElementSpec {
        id: "header.issuer_mark",
        name: "发文机关标志",
        part: Part::Header,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::IssuerMark),
        font_size_pt: None,
        color: RED,
        align: Alignment::Center,
        position: "居中排布, 上边缘至版心上边缘 35 mm",
        spacing: "不涉及",
        rules: "由发文机关全称或规范化简称加“文件”二字组成, 也可只用机关名称; 推荐小标宋体字, 红色; 以醒目、美观、庄重为原则",
        clause: "7.2.4",
    },
    ElementSpec {
        id: "header.joint_issuer_mark",
        name: "联合行文发文机关标志",
        part: Part::Header,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::IssuerMark),
        font_size_pt: None,
        color: RED,
        align: Alignment::Center,
        position: "主办机关名称排列在前; “文件”二字置于发文机关名称右侧, 以联署发文机关名称为准上下居中",
        spacing: "不涉及",
        rules: "联合行文时, 如需同时标注联署发文机关名称, 按上述规则排列",
        clause: "7.2.4",
    },
    ElementSpec {
        id: "header.doc_number",
        name: "发文字号",
        part: Part::Header,
        requirement: Requirement::Required,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Center,
        position: "发文机关标志下空二行居中; 上行文时居左空一字, 与最后一个签发人姓名同一行",
        spacing: "发文机关标志下空二行",
        rules: "年份全称并用六角括号“〔〕”; 发文顺序号用阿拉伯数字, 不加“第”字, 不编虚位, 后加“号”字",
        clause: "7.2.5",
    },
    ElementSpec {
        id: "header.signers",
        name: "签发人",
        part: Part::Header,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Right,
        position: "居右空一字, 与发文字号同在发文机关标志下空二行位置",
        spacing: "发文机关标志下空二行",
        rules: "“签发人”三字用 3 号仿宋体字, 姓名用 3 号楷体字; 多个签发人每行一般排两个, 回行与上一行第一个签发人姓名对齐",
        clause: "7.2.6",
    },
    ElementSpec {
        id: "header.red_line",
        name: "版头中的分隔线",
        part: Part::Header,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: RED,
        align: Alignment::Center,
        position: "发文字号之下 4 mm 处, 与版心等宽, 居中",
        spacing: "发文字号下 4 mm",
        rules: "红色分隔线",
        clause: "7.2.7",
    },
    // ------------------------------------------------------------------
    // 7.3 主体
    // ------------------------------------------------------------------
    ElementSpec {
        id: "body.title",
        name: "标题",
        part: Part::Body,
        requirement: Requirement::Required,
        font: Some(FontSlot::Title),
        font_size_pt: Some(22.0),
        color: BLACK,
        align: Alignment::Center,
        position: "红色分隔线下空二行, 分一行或多行居中排布",
        spacing: "红色分隔线下空二行",
        rules: "2 号小标宋体字; 回行时词意完整、排列对称、长短适宜、间距恰当; 排列使用梯形或菱形",
        clause: "7.3.1",
    },
    ElementSpec {
        id: "body.addressee",
        name: "主送机关",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::TopLeft,
        position: "标题下空一行, 居左顶格; 回行仍顶格; 最后一个机关名称后标全角冒号",
        spacing: "标题下空一行",
        rules: "名称过多导致首页不能显示正文时, 移至版记(见 7.4.2)",
        clause: "7.3.2",
    },
    ElementSpec {
        id: "body.text",
        name: "正文",
        part: Part::Body,
        requirement: Requirement::Required,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Justify,
        position: "主送机关下一行, 首页必须显示正文",
        spacing: "自然段左空二字, 回行顶格",
        rules: "一般用 3 号仿宋体字; 每面 22 行×每行 28 字并撑满版心",
        clause: "7.3.3",
    },
    ElementSpec {
        id: "body.hierarchy",
        name: "结构层次序数",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Left,
        position: "正文内依次使用“一、”“(一)”“1.”“(1)”标注",
        spacing: "随正文",
        rules: "第一层用黑体字, 第二层用楷体字, 第三层和第四层用仿宋体字标注",
        clause: "7.3.3",
    },
    ElementSpec {
        id: "body.attachment_note",
        name: "附件说明",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Left,
        position: "正文下空一行左空二字; 多个附件用阿拉伯数字标注顺序号(如“附件:1.×××”)",
        spacing: "正文下空一行",
        rules: "附件名称后不加标点符号; 名称较长回行时与上一行附件名称首字对齐",
        clause: "7.3.4",
    },
    ElementSpec {
        id: "body.signature_with_seal",
        name: "发文机关署名、成文日期和印章(加盖印章)",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Right,
        position: "成文日期一般右空四字; 发文机关署名在成文日期之上、以成文日期为准居中; 印章端正、居中下压署名和日期",
        spacing: "印章顶端上距正文(或附件说明)一行之内",
        rules: "印章用红色, 不得出现空白印章; 单一机关与联合行文分别排版; 联合行文印章排列整齐、互不相交或相切, 每排两端不超出版心",
        clause: "7.3.5.1",
    },
    ElementSpec {
        id: "body.signature_without_seal",
        name: "发文机关署名、成文日期(不加盖印章)",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Right,
        position: "正文(或附件说明)下空一行右空二字编排署名; 下一行编排成文日期, 首字比署名首字右移二字; 日期长于署名时日期右空二字并相应增加署名右空字数",
        spacing: "正文(或附件说明)下空一行",
        rules: "联合行文时先排主办机关署名, 其余依次向下编排",
        clause: "7.3.5.2",
    },
    ElementSpec {
        id: "body.signature_seal",
        name: "签发人签名章",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: RED,
        align: Alignment::Right,
        position: "正文(或附件说明)下空二行右空四字加盖签名章; 签名章左空二字标注签发人职务, 以签名章为准上下居中; 签名章下空一行右空四字编排成文日期",
        spacing: "正文(或附件说明)下空二行; 签名章下空一行",
        rules: "签发人职务标注全称; 联合行文时每行只编排一个机关的签发人职务和签名章, 与主办机关上下对齐; 签名章一般用红色",
        clause: "7.3.5.3",
    },
    ElementSpec {
        id: "body.date_digits",
        name: "成文日期中的数字",
        part: Part::Body,
        requirement: Requirement::Required,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Right,
        position: "随成文日期",
        spacing: "不涉及",
        rules: "用阿拉伯数字将年、月、日标全, 年份标全称, 月、日不编虚位(即 1 不编为 01)",
        clause: "7.3.5.4",
    },
    ElementSpec {
        id: "body.special_spacing",
        name: "特殊情况说明(调整行距字距)",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "所剩空白不能容下印章/签名章/成文日期时",
        spacing: "可通过调整行距、字距解决",
        rules: "排版后所剩空白处不能容下印章或签发人签名章、成文日期时, 可采取调整行距、字距的措施解决",
        clause: "7.3.5.5",
    },
    ElementSpec {
        id: "body.annotation",
        name: "附注",
        part: Part::Body,
        requirement: Requirement::Optional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::Left,
        position: "成文日期下一行, 居左空二字, 加圆括号",
        spacing: "成文日期下一行",
        rules: "如“（此件公开发布）”",
        clause: "7.3.6",
    },
    ElementSpec {
        id: "body.attachment",
        name: "附件",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "另面编排, 在版记之前, 与公文正文一起装订",
        spacing: "另起一面",
        rules: "附件格式要求同正文; 附件顺序号和标题与附件说明一致",
        clause: "7.3.7",
    },
    ElementSpec {
        id: "body.attachment_header",
        name: "附件标注与附件标题",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Heiti),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::TopLeft,
        position: "“附件”二字及附件顺序号用 3 号黑体字顶格编排在版心左上角第一行; 附件标题居中编排在版心第三行",
        spacing: "附件标题在版心第三行",
        rules: "“附件”二字及顺序号与附件说明的表述一致",
        clause: "7.3.7",
    },
    ElementSpec {
        id: "body.attachment_standalone",
        name: "附件与正文不一起装订时的标注",
        part: Part::Body,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Body),
        font_size_pt: Some(16.0),
        color: BLACK,
        align: Alignment::TopLeft,
        position: "附件左上角第一行顶格编排公文的发文字号, 其后标注“附件”二字及附件顺序号",
        spacing: "第一行",
        rules: "仅在附件与正文不能一起装订时使用",
        clause: "7.3.7",
    },
    // ------------------------------------------------------------------
    // 7.4 版记
    // ------------------------------------------------------------------
    ElementSpec {
        id: "record.separator_lines",
        name: "版记中的分隔线",
        part: Part::Record,
        requirement: Requirement::Conditional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::Center,
        position: "与版心等宽; 首条分隔线位于版记第一个要素之上, 末条分隔线与公文最后一面的版心下边缘重合",
        spacing: "首末条粗线推荐 0.35 mm; 中间细线推荐 0.25 mm",
        rules: "首条和末条用粗线, 中间用细线",
        clause: "7.4.1",
    },
    ElementSpec {
        id: "record.cc",
        name: "抄送机关",
        part: Part::Record,
        requirement: Requirement::Optional,
        font: Some(FontSlot::Record),
        font_size_pt: Some(14.0),
        color: BLACK,
        align: Alignment::Left,
        position: "印发机关和印发日期之上一行, 左右各空一字; “抄送”二字后加全角冒号和机关名称",
        spacing: "左右各空一字",
        rules: "4 号仿宋体字; 回行与冒号后首字对齐; 最后一个机关名称后标句号",
        clause: "7.4.2",
    },
    ElementSpec {
        id: "record.addressee_moved",
        name: "主送机关移至版记",
        part: Part::Record,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Record),
        font_size_pt: Some(14.0),
        color: BLACK,
        align: Alignment::Left,
        position: "编排方法同抄送机关, “抄送”改为“主送”; 既有主送又有抄送时置于抄送之上一行, 之间不加分隔线",
        spacing: "左右各空一字",
        rules: "仅在主送机关名称过多导致公文首页不能显示正文时使用",
        clause: "7.4.2",
    },
    ElementSpec {
        id: "record.printing",
        name: "印发机关和印发日期",
        part: Part::Record,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::Record),
        font_size_pt: Some(14.0),
        color: BLACK,
        align: Alignment::Left,
        position: "末条分隔线之上; 印发机关左空一字, 印发日期右空一字",
        spacing: "末条分隔线之上",
        rules: "4 号仿宋体字; 日期用阿拉伯数字将年月日标全, 年份全称, 月日不编虚位, 后加“印发”二字",
        clause: "7.4.3",
    },
    ElementSpec {
        id: "record.other_elements",
        name: "版记中的其他要素",
        part: Part::Record,
        requirement: Requirement::Optional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::Left,
        position: "印发机关和印发日期之上, 与印发机关和印发日期用一条细分隔线隔开",
        spacing: "细分隔线",
        rules: "版记中如有其他要素, 用细分隔线隔开",
        clause: "7.4.3",
    },
    // ------------------------------------------------------------------
    // 7.5 页码
    // ------------------------------------------------------------------
    ElementSpec {
        id: "page.number",
        name: "页码",
        part: Part::PageNumber,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::PageNumber),
        font_size_pt: Some(14.0),
        color: BLACK,
        align: Alignment::Right,
        position: "版心下边缘之下; 数字左右各一条一字线, 一字线上距版心下边缘 7 mm; 单页码居右空一字, 双页码居左空一字",
        spacing: "一字线上距版心下边缘 7 mm",
        rules: "4 号半角宋体阿拉伯数字; 版记页前有空白页的, 空白页和版记页均不编排页码; 附件与正文一起装订时页码连续编排",
        clause: "7.5",
    },
    // ------------------------------------------------------------------
    // 第 8 章 横排表格
    // ------------------------------------------------------------------
    ElementSpec {
        id: "table.landscape",
        name: "公文中的横排表格",
        part: Part::Table,
        requirement: Requirement::Conditional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "A4 纸型表格横排时, 页码位置与其他页码一致; 单页码表头在订口一边, 双页码表头在切口一边",
        spacing: "不涉及",
        rules: "横排表格所在页的页码位置保持不变, 表头方向按单双页翻转",
        clause: "8",
    },
    // ------------------------------------------------------------------
    // 第 9 章 计量单位、标点符号和数字
    // ------------------------------------------------------------------
    ElementSpec {
        id: "content.units",
        name: "计量单位、标点符号和数字用法",
        part: Part::ContentRules,
        requirement: Requirement::Required,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "不涉及",
        spacing: "不涉及",
        rules: "计量单位符合 GB 3100、GB 3101、GB 3102(所有部分); 标点符号符合 GB/T 15834; 数字用法符合 GB/T 15835",
        clause: "9",
    },
    // ------------------------------------------------------------------
    // 第 10 章 特定格式
    // ------------------------------------------------------------------
    ElementSpec {
        id: "special.letter",
        name: "信函格式",
        part: Part::Special,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::IssuerMark),
        font_size_pt: None,
        color: RED,
        align: Alignment::Center,
        position: "发文机关标志上边缘至上页边 30 mm; 标志下 4 mm 红色双线(上粗下细), 下页边 20 mm 处红色双线(上细下粗), 线长均为 170 mm 居中; 份号/密级/紧急程度在第一条双线下顶格; 发文字号顶格居版心右边缘; 标题与其上最后一个要素相距二行",
        spacing: "份号/密级/紧急程度/发文字号与第一条双线距离为 3 号汉字高度的 7/8",
        rules: "首页不显示页码; 版记不加印发机关和印发日期、分隔线, 位于最后一面版心最下方; 联合行文使用主办机关标志",
        clause: "10.1",
    },
    ElementSpec {
        id: "special.order",
        name: "命令(令)格式",
        part: Part::Special,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::IssuerMark),
        font_size_pt: None,
        color: RED,
        align: Alignment::Center,
        position: "发文机关标志由发文机关全称加“命令”或“令”字组成, 上边缘至版心上边缘 20 mm; 标志下空二行居中编排令号; 令号下空二行编排正文",
        spacing: "标志下空二行; 令号下空二行",
        rules: "推荐小标宋体字, 红色; 签发人职务、签名章和成文日期编排见 7.3.5.3",
        clause: "10.2",
    },
    ElementSpec {
        id: "special.minutes",
        name: "纪要格式",
        part: Part::Special,
        requirement: Requirement::Conditional,
        font: Some(FontSlot::IssuerMark),
        font_size_pt: None,
        color: RED,
        align: Alignment::Center,
        position: "纪要标志由“×××××纪要”组成, 上边缘至版心上边缘 35 mm; 出席人员名单在正文或附件说明下空一行左空二字",
        spacing: "正文或附件说明下空一行",
        rules: "推荐小标宋体字, 红色; “出席”二字为 3 号黑体, 冒号后用 3 号仿宋标注单位、姓名, 回行与冒号后首字对齐; 请假/列席人员另起一行, 编排方法同出席; 纪要格式可根据实际制定",
        clause: "10.3",
    },
    // ------------------------------------------------------------------
    // 第 11 章 式样
    // ------------------------------------------------------------------
    ElementSpec {
        id: "sample.figures",
        name: "公文式样(图 1~图 12)",
        part: Part::Sample,
        requirement: Requirement::Optional,
        font: None,
        font_size_pt: None,
        color: BLACK,
        align: Alignment::NotApplicable,
        position: "见图 1~图 12: A4 页边及版心尺寸、公文首页版式、联合行文首页版式 1/2、公文末页版式 1/2、联合行文末页版式 1/2、附件说明页版式、带附件公文末页版式、信函格式首页、命令(令)格式首页",
        spacing: "不涉及",
        rules: "式样仅作示意; 注: 版心实线框仅为示意, 印制公文时并不印出",
        clause: "11",
    },
];

/// 按 id 查要素.
pub fn find(id: &str) -> Option<&'static ElementSpec> {
    ELEMENTS.iter().find(|e| e.id == id)
}

/// 按部分分组.
pub fn by_part(part: Part) -> impl Iterator<Item = &'static ElementSpec> {
    ELEMENTS.iter().filter(move |e| e.part == part)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn element_ids_are_unique() {
        let mut seen = HashSet::new();
        for element in ELEMENTS {
            assert!(seen.insert(element.id), "重复的要素 id: {}", element.id);
        }
    }

    #[test]
    fn registry_covers_every_chapter_of_the_standard() {
        let parts: HashSet<Part> = ELEMENTS.iter().map(|e| e.part).collect();
        for part in [
            Part::Paper,
            Part::Layout,
            Part::Printing,
            Part::Header,
            Part::Body,
            Part::Record,
            Part::PageNumber,
            Part::Special,
            Part::Table,
            Part::ContentRules,
            Part::Sample,
        ] {
            assert!(parts.contains(&part), "缺少 {:?} 部分的要素", part);
        }
    }

    #[test]
    fn every_clause_is_non_empty() {
        for element in ELEMENTS {
            assert!(!element.clause.is_empty(), "{} 缺少条款号", element.id);
            assert!(!element.rules.is_empty(), "{} 缺少规则", element.id);
        }
    }

    #[test]
    fn font_bearing_elements_have_sizes_except_issuer_mark() {
        for element in ELEMENTS {
            if element.font.is_some() && element.font != Some(FontSlot::IssuerMark) {
                assert!(element.font_size_pt.is_some(), "{} 缺少字号", element.id);
            }
        }
    }

    #[test]
    fn find_round_trips() {
        for element in ELEMENTS {
            assert_eq!(find(element.id).map(|e| e.id), Some(element.id));
        }
        assert!(find("no.such.element").is_none());
    }
}

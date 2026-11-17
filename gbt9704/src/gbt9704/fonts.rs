//! 字体槽位与 GB/T 9704-2012 的字体要求.
//!
//! 标准不是"指定字体文件", 而是指定字体族 + 字号 + 颜色(见 5.2.2/7.x/10.x).
//! 生成器内部用稳定的槽位(slot)名引用字体, 具体字体文件在阶段 2(PDF) 才需要嵌入,
//! 由 `docs/fonts.md` 与 `fonts-manifest.json` 提供官方字体/开源替代的映射.
//!
//! 授权提示: 方正小标宋简体、仿宋_GB2312、楷体_GB2312 等为商业/系统字体, 不随项目分发.

use super::geometry::{BLACK, Color, RED};

/// 字体槽位: 公文里出现的每一类字体需求.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FontSlot {
    /// 2 号小标宋体: 标题 (7.3.1).
    Title,
    /// 3 号仿宋体: 正文与一般要素、版头发文字号、签发人三字、附件说明等 (5.2.2/7.2.5/7.3.3).
    Body,
    /// 3 号黑体: 密级和保密期限、紧急程度、一级层次标题、"附件"二字 (7.2.2/7.2.3/7.3.3/7.3.7).
    Heiti,
    /// 3 号楷体: 签发人姓名、二级层次标题、信函标题? (7.2.6/7.3.3).
    Kaiti,
    /// 4 号仿宋: 抄送机关、印发机关和印发日期 (7.4.2/7.4.3).
    Record,
    /// 4 号半角宋体: 页码数字 (7.5).
    PageNumber,
    /// 小标宋(红色, 字号由版式设计决定): 发文机关标志/信函标志/命令(令)标志/纪要标志 (7.2.4/10.1/10.2/10.3).
    IssuerMark,
}

impl FontSlot {
    /// 稳定 id, 用于 JSON/XML/配置.
    pub fn id(self) -> &'static str {
        match self {
            FontSlot::Title => "title",
            // 与 fonts-manifest.json 的 slot 名保持一致; 旧名 "body" 由 from_id 兼容.
            FontSlot::Body => "body_fangsong",
            FontSlot::Heiti => "heiti",
            FontSlot::Kaiti => "kaiti",
            FontSlot::Record => "record",
            FontSlot::PageNumber => "page_number",
            FontSlot::IssuerMark => "issuer_mark",
        }
    }

    /// 标准中的字体族名称 (不是文件名).
    pub fn standard_family(self) -> &'static str {
        match self {
            FontSlot::Title => "小标宋体",
            FontSlot::Body => "仿宋体",
            FontSlot::Heiti => "黑体",
            FontSlot::Kaiti => "楷体",
            FontSlot::Record => "仿宋体",
            FontSlot::PageNumber => "宋体(半角阿拉伯数字)",
            FontSlot::IssuerMark => "小标宋体",
        }
    }

    /// 与 `fonts-manifest.json` 对齐的槽位名(旧 id "body" 仍可作为输入别名).
    pub fn manifest_id(self) -> &'static str {
        self.id()
    }

    /// 按 id 反查, 供 JSON 输入使用.
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "title" => Some(FontSlot::Title),
            "body" | "body_fangsong" | "fangsong" => Some(FontSlot::Body),
            "heiti" => Some(FontSlot::Heiti),
            "kaiti" => Some(FontSlot::Kaiti),
            "record" => Some(FontSlot::Record),
            "page_number" => Some(FontSlot::PageNumber),
            "issuer_mark" => Some(FontSlot::IssuerMark),
            _ => None,
        }
    }
}

/// 一条字体要求: 标准原文槽位 + 字号 + 颜色 + 出处.
#[derive(Debug, Clone, Copy)]
pub struct FontRequirement {
    pub slot: FontSlot,
    /// 标准措辞, 例如 "2号小标宋体字".
    pub standard_text: &'static str,
    /// 字号; 发文机关标志等标准未规定字号, 为 None.
    pub size_pt: Option<f64>,
    pub color: Color,
    pub usage: &'static str,
    pub clause: &'static str,
}

/// 全部字体要求.
pub const REQUIREMENTS: &[FontRequirement] = &[
    FontRequirement {
        slot: FontSlot::Title,
        standard_text: "2号小标宋体字",
        size_pt: Some(22.0),
        color: BLACK,
        usage: "标题",
        clause: "7.3.1",
    },
    FontRequirement {
        slot: FontSlot::Body,
        standard_text: "3号仿宋体字",
        size_pt: Some(16.0),
        color: BLACK,
        usage: "正文及如无特殊说明的各要素; 发文字号; “签发人”三字; 发布日期以外的一般文字",
        clause: "5.2.2/7.2.5/7.2.6/7.3.3",
    },
    FontRequirement {
        slot: FontSlot::Heiti,
        standard_text: "3号黑体字",
        size_pt: Some(16.0),
        color: BLACK,
        usage: "密级和保密期限、紧急程度、第一层结构层次序数、附件标注",
        clause: "7.2.2/7.2.3/7.3.3/7.3.7",
    },
    FontRequirement {
        slot: FontSlot::Kaiti,
        standard_text: "3号楷体字",
        size_pt: Some(16.0),
        color: BLACK,
        usage: "签发人姓名、第二层结构层次序数",
        clause: "7.2.6/7.3.3",
    },
    FontRequirement {
        slot: FontSlot::Record,
        standard_text: "4号仿宋体字",
        size_pt: Some(14.0),
        color: BLACK,
        usage: "抄送机关、印发机关和印发日期、移入版记的主送机关",
        clause: "7.4.2/7.4.3",
    },
    FontRequirement {
        slot: FontSlot::PageNumber,
        standard_text: "4号半角宋体阿拉伯数字",
        size_pt: Some(14.0),
        color: BLACK,
        usage: "页码数字",
        clause: "7.5",
    },
    FontRequirement {
        slot: FontSlot::IssuerMark,
        standard_text: "推荐使用小标宋体字",
        size_pt: None,
        color: RED,
        usage: "发文机关标志、信函标志、命令(令)标志、纪要标志",
        clause: "7.2.4/10.1/10.2/10.3",
    },
];

/// 一条可用字体(官方或开源替代), 供阶段 0 字体寻找与阶段 2 嵌入使用.
#[derive(Debug, Clone, Copy)]
pub struct FontCandidate {
    pub slot: FontSlot,
    /// 字体家族名 (PostScript/OpenType family 的常见写法).
    pub family: &'static str,
    /// 许可证标识.
    pub license: &'static str,
    /// 是否可自由再分发/嵌入.
    pub redistributable: bool,
    /// 与标准字形的差距/注意事项.
    pub note: &'static str,
}

/// 已知/候选字体. 具体文件名、SHA256、下载 URL 见 `fonts-manifest.json` 与 `docs/fonts.md`.
///
/// 授权结论:
/// - Noto CJK / 霞鹜文楷 / 朱雀仿宋: SIL OFL 1.1, 可嵌入/可随软件分发;
/// - Fandol: GPL-3.0-or-later WITH GPL font exception, 仅作本地下载的回退字体, 不随项目分发;
/// - 方正小标宋简体 / 仿宋_GB2312 / 楷体_GB2312 / 黑体 / 宋体: 商业或系统字体, 需用户自备授权.
pub const CANDIDATES: &[FontCandidate] = &[
    FontCandidate {
        slot: FontSlot::Title,
        family: "Noto Serif CJK SC Bold",
        license: "SIL OFL 1.1",
        redistributable: true,
        note: "首选开源近似(文件 NotoSerifCJKsc-Bold.otf); 无开源“小标宋”等价字族, 严格要求需自备方正小标宋简体",
    },
    FontCandidate {
        slot: FontSlot::Title,
        family: "FandolSong-Bold",
        license: "GPL-3.0-or-later WITH font exception",
        redistributable: false,
        note: "回退(文件 FandolSong-Bold.otf); 粗宋骨架, 字形与小标宋仍有差距",
    },
    FontCandidate {
        slot: FontSlot::Body,
        family: "FandolFang",
        license: "GPL-3.0-or-later WITH font exception",
        redistributable: false,
        note: "首选本地替代(文件 FandolFang-Regular.otf, GB2312 全覆盖); 观感接近仿宋_GB2312",
    },
    FontCandidate {
        slot: FontSlot::Body,
        family: "Zhuque Fangsong (朱雀仿宋)",
        license: "SIL OFL 1.1",
        redistributable: true,
        note: "可分发备选(文件 ZhuqueFangsong-Regular.ttf); 风格化技术预览, 字表/全角数字覆盖需实测",
    },
    FontCandidate {
        slot: FontSlot::Heiti,
        family: "Noto Sans CJK SC",
        license: "SIL OFL 1.1",
        redistributable: true,
        note: "首选(文件 NotoSansCJKsc-Regular.otf); 标准要求“黑体”即无衬线, 不是加粗宋体",
    },
    FontCandidate {
        slot: FontSlot::Heiti,
        family: "FandolHei",
        license: "GPL-3.0-or-later WITH font exception",
        redistributable: false,
        note: "回退(文件 FandolHei-Regular.otf)",
    },
    FontCandidate {
        slot: FontSlot::Kaiti,
        family: "LXGW WenKai (霞鹜文楷)",
        license: "SIL OFL 1.1 (保留名称霞鹜/LXGW)",
        redistributable: true,
        note: "首选(文件 LXGWWenKai-Regular.ttf); 源自 Klee One, 少量日式字形, 严格中易楷体字形需自备楷体_GB2312",
    },
    FontCandidate {
        slot: FontSlot::Kaiti,
        family: "FandolKai",
        license: "GPL-3.0-or-later WITH font exception",
        redistributable: false,
        note: "回退(文件 FandolKai-Regular.otf)",
    },
    FontCandidate {
        slot: FontSlot::PageNumber,
        family: "Noto Serif CJK SC",
        license: "SIL OFL 1.1",
        redistributable: true,
        note: "首选(文件 NotoSerifCJKsc-Regular.otf); 必须用半角数字 U+0030–U+0039, 不用全角０–９",
    },
    FontCandidate {
        slot: FontSlot::PageNumber,
        family: "FandolSong",
        license: "GPL-3.0-or-later WITH font exception",
        redistributable: false,
        note: "回退(文件 FandolSong-Regular.otf); 半角数字比例宽, 需确认一字线对齐",
    },
    FontCandidate {
        slot: FontSlot::Record,
        family: "FandolFang",
        license: "GPL-3.0-or-later WITH font exception",
        redistributable: false,
        note: "4 号仿宋, 与正文共用 FandolFang-Regular.otf",
    },
    FontCandidate {
        slot: FontSlot::IssuerMark,
        family: "Noto Serif CJK SC Bold",
        license: "SIL OFL 1.1",
        redistributable: true,
        note: "红色标志首选的跨平台近似; 标准未规定字号, 建议 22–48 pt 自适应(默认约 36 pt), 单行不超过版心 156 mm",
    },
    FontCandidate {
        slot: FontSlot::IssuerMark,
        family: "方正小标宋简体 (FZXiaoBiaoSong-B05S)",
        license: "商业",
        redistributable: false,
        note: "严格合规字形; 需用户自备授权, 不随项目分发",
    },
];

/// 按槽位取要求.
pub fn requirement(slot: FontSlot) -> Option<&'static FontRequirement> {
    REQUIREMENTS.iter().find(|r| r.slot == slot)
}

/// 按槽位取所有候选字体.
pub fn candidates_for(slot: FontSlot) -> impl Iterator<Item = &'static FontCandidate> {
    CANDIDATES.iter().filter(move |c| c.slot == slot)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_slot_has_requirement_and_candidate() {
        for slot in [
            FontSlot::Title,
            FontSlot::Body,
            FontSlot::Heiti,
            FontSlot::Kaiti,
            FontSlot::Record,
            FontSlot::PageNumber,
            FontSlot::IssuerMark,
        ] {
            assert!(requirement(slot).is_some(), "缺少字体要求: {:?}", slot);
            assert!(
                candidates_for(slot).next().is_some(),
                "缺少字体候选: {:?}",
                slot
            );
        }
    }

    #[test]
    fn slot_ids_are_unique_and_round_trip() {
        let slots = [
            FontSlot::Title,
            FontSlot::Body,
            FontSlot::Heiti,
            FontSlot::Kaiti,
            FontSlot::Record,
            FontSlot::PageNumber,
            FontSlot::IssuerMark,
        ];
        for (index, slot) in slots.iter().enumerate() {
            assert_eq!(FontSlot::from_id(slot.id()), Some(*slot));
            for other in &slots[index + 1..] {
                assert_ne!(slot.id(), other.id());
            }
        }
    }
}

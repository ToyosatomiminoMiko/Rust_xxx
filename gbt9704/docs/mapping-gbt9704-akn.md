# GB/T 9704-2012 → Akoma Ntoso 3.1 全量映射表

> 本文件由 `cargo run -p gbt9704 -- mapping` 生成; 修改请改 `src/mapping.rs`。

## 1. 目标标准

- 公文格式: **GB/T 9704-2012《党政机关公文格式》** (2012-06-29 发布, 2012-07-01 实施, 代替 GB/T 9704-1999)
- Akoma Ntoso: **Akoma Ntoso Version 2.0, Part 2: Specifications of AKN 3.1** (OASIS LegalDocML, OASIS Standard, 2026-10-02 生效)
- 命名空间: 默认口径 A `http://docs.oasis-open.org/legaldocml/ns/akn/3.0` + 根元素 `version="3.1"` (官方 XSD 可校验); 可选口径 B `.../ns/akn/3.1` (OASIS v2.0 命名空间文档声明, 官方未随发对应 XSD, 需本地 adapted schema)
- 扩展命名空间: `urn:gbt9704:ext:1.0`, 只出现在 `akn:proprietary` / `akn:presentation` 内部

## 2. 映射类别

| 类别 | 英文 id | 含义 | 数量 |
|---|---|---|---|
| 直接对应 | `direct` | AKN 原生元素/属性可精确表达 | 6 |
| 语义近似 | `near_fit` | AKN 有相近元素, 但语义有偏移, 需讨论 | 14 |
| 扩展承载 | `extension` | AKN 无原生元素, 使用 proprietary/presentation 官方扩展点承载 | 8 |
| 无对应(版式/印制) | `no_fit` | AKN 语义模型无对应, 由渲染/印后工艺执行 | 16 |

合计: **44** 项标准要素, 全部有明确去向; 其中存在语义偏移/无原生对应(**需讨论**)的有 **38** 项。

## 3. 全量映射表

### 3.1 用纸

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 公文用纸技术指标 (`paper.technical`) | 4 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:element[paper.technical] | 纸张物理指标是印后/材料要求, 不属于文档语义; 仅作为呈现参数存档 |

### 3.2 版面

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 幅面尺寸(A4) (`paper.a4`) | 5.1 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:page | 幅面是 manifestation 的物理属性; AKN 无页面尺寸元素 |
| 页边与版心尺寸 (`layout.margins`) | 5.2.1 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:margins + gbt:text-area | 页边/版心属于版面几何; AKN 只描述结构, 不描述版心 |
| 字体和字号(默认) (`layout.default_font`) | 5.2.2 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:font[slot=body] | 字体字号是呈现属性; AKN 的文本内容不带字体 |
| 行数和字数 (`layout.grid`) | 5.2.3 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:text-area + gbt:grid | 22 行×28 字是版面网格; AKN 无行数字数概念 |
| 字、行(术语定义) (`layout.units`) | 3.1/3.2 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:grid(line-unit-pt/char-pitch-pt) + presentation/gbt:element[layout.units] | “字/行”是排版距离单位; 术语定义值(16 pt/30 pt)与 22 行×28 字网格值(15.79 pt/28.99 pt)是两套常量, AKN 只能存参数 |
| 文字的颜色 (`layout.text_color`) | 5.2.4 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:colors | 文字颜色是呈现属性 |

### 3.3 印制装订

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 制版要求 (`printing.plate`) | 6.1 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:element[printing.plate] | 制版质量要求不属于文档内容 |
| 印刷要求 (`printing.ink`) | 6.2 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:colors(黑色/红色油墨) | 印刷油墨色值(CMY K)属于印后工艺; RGB/CMYK 由阶段 2 处理 |
| 装订要求 (`printing.binding`) | 6.3 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:element[printing.binding] | 装订方式/订位不属于文档结构 |

### 3.4 版头

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 份号 (`header.copy_number`) | 7.2.1 | 扩展承载 | preface/p[@gbt:role='copyNumber'] + gbt:digits | AKN 无份号元素; 理论上可借用 FRBR Item 层标识同一 expression 的不同副本, 但 AKN 没有 item 级编号元素 |
| 密级和保密期限 (`header.secrecy`) | 7.2.2 | 语义近似 | meta/classification/keyword(dictionary=#gbt_dict_secrecy) + preface/p[@gbt:role='secrecy'] | classification/keyword 的语义是主题分类/受控词表, 不是保密等级; 密级和保密期限被当作关键词承载, 语义偏移 |
| 紧急程度 (`header.urgency`) | 7.2.3 | 语义近似 | meta/classification/keyword(dictionary=#gbt_dict_urgency) + preface/p[@gbt:role='urgency'] | 紧急程度同样不是主题关键词; 若 AKN 未来提供 docUrgency 类元素应迁移 |
| 发文机关标志 (`header.issuer_mark`) | 7.2.4 | 语义近似 | FRBRauthor(TLCOrganization) + preface/p[@gbt:role='issuerMark'] + presentation/gbt:font[issuer_mark] | 发文机关标志 = 机关名称(语义) + 红色小标宋/35 mm 版头位置(视觉); AKN 只表达机关, 视觉部分只能进扩展 |
| 联合行文发文机关标志 (`header.joint_issuer_mark`) | 7.2.4 | 语义近似 | 多个 FRBRauthor(TLCOrganization) + preface/p[@gbt:role='issuerMark'] + presentation | 联合行文“主办机关在前、文件二字在右、上下居中”是版式规则; AKN 用多个并列 author 表达机关集合, 不表达排布 |
| 发文字号 (`header.doc_number`) | 7.2.5 | 直接对应 | preface/p/docNumber + FRBRWork/FRBRnumber + FRBRWork/FRBRuri(工作层 IRI 含发文年份/序号) | 无 |
| 签发人 (`header.signers`) | 7.2.6 | 语义近似 | workflow/step(@date,@by,@as) + preface/p/docProponent + references/TLCPerson,TLC Role | AKN 无 docSignatory 元素; 签发人既是签名者(workflow 步骤), 又近似 docProponent(提议人), 语义不完全等价 |
| 版头中的分隔线 (`header.red_line`) | 7.2.7 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:element[header.red_line] | 红色分隔线是版头视觉分隔, AKN 无版头/分隔线概念 |

### 3.5 主体

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 标题 (`body.title`) | 7.3.1 | 直接对应 | preface/p/docTitle + FRBRWork/FRBRname; 版式在 presentation | 无 |
| 主送机关 (`body.addressee`) | 7.3.2 | 扩展承载 | preface/p[@gbt:role='addressee'](纯文本); 无 AKN 元素 | AKN 3.1 没有主送机关/收文者元素(docAddressee 不存在); 只能作为 preface 普通段落 + 扩展属性, 属重点待讨论项 |
| 正文 (`body.text`) | 7.3.3 | 直接对应 | mainBody/p(自然段) | 无 |
| 结构层次序数 (`body.hierarchy`) | 7.3.3 | 直接对应 | mainBody/hcontainer[@name='level-1'..'level-4'](num,heading,intro,content,wrapUp) 嵌套 | 无 |
| 附件说明 (`body.attachment_note`) | 7.3.4 | 扩展承载 | conclusions/p[@gbt:role='attachmentNote'] + attachments/attachment 清单 | AKN 无“附件说明”元素; 附件说明是正文对附件的引用清单, 用 conclusions 段落 + 扩展属性表达, 附件实体在 attachments |
| 发文机关署名、成文日期和印章(加盖印章) (`body.signature_with_seal`) | 7.3.5.1 | 语义近似 | conclusions/p/signature + FRBRauthor; 印章图形/位置在扩展 | AKN 的 signature 是文本签名标识, 不表达红色印章图形、下压位置与“不得空白印章”等版式要求; 印章是图形资源 |
| 发文机关署名、成文日期(不加盖印章) (`body.signature_without_seal`) | 7.3.5.2 | 语义近似 | conclusions/p/signature + FRBRauthor | 署名与日期的相对位置(右空二字/首字右移二字)是版式, AKN 不表达 |
| 签发人签名章 (`body.signature_seal`) | 7.3.5.3 | 语义近似 | conclusions/p/signature + workflow(签发人) + 签名章占位扩展 | 签名章是图形资源; 签发人职务、签名章、成文日期的上下居中关系属版式; AKN 只能表达签发人身份与日期 |
| 成文日期中的数字 (`body.date_digits`) | 7.3.5.4 | 直接对应 | conclusions/p/date(@date=ISO) + FRBRdate; “年月日不编虚位”是呈现规则, 由渲染层执行 | 无 |
| 特殊情况说明(调整行距字距) (`body.special_spacing`) | 7.3.5.5 | 无对应(版式/印制) | 无 AKN 语义; 由阶段 2 排版算法执行, 参数在 presentation | “调整行距、字距”是排版算法, AKN 无法表达 |
| 附注 (`body.annotation`) | 7.3.6 | 语义近似 | meta/notes/note(@marker,@placement) + conclusions/p[@gbt:role='annotation'] | AKN note 的语义是脚注/尾注/页边注; 公文附注是成文日期后的说明(如“此件公开发布”), 语义接近但不完全一致 |
| 附件 (`body.attachment`) | 7.3.7 | 直接对应 | attachments/attachment/doc(独立 FRBR 链) | 无 |
| 附件标注与附件标题 (`body.attachment_header`) | 7.3.7 | 扩展承载 | attachment/doc/preface/p[@gbt:role='attachmentLabel'\|'attachmentTitle'] + docTitle | “附件”二字与顺序号、附件标题在版心第三行是版式; AKN 用嵌套 doc 的 preface 承载说明文字 |
| 附件与正文不一起装订时的标注 (`body.attachment_standalone`) | 7.3.7 | 扩展承载 | attachment/doc/preface/p[@gbt:role='attachmentStandaloneLabel'] + gbt:attachment-standalone | AKN 无“附件与正文分开装订时以发文字号标注”的概念 |

### 3.6 版记

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 版记中的分隔线 (`record.separator_lines`) | 7.4.1 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:record-lines | 版记分隔线(粗 0.35/细 0.25 mm)是纯版式 |
| 抄送机关 (`record.cc`) | 7.4.2 | 扩展承载 | proprietary/gbt:gbt9704/gbt:record/gbt:cc(备选: conclusions/container[@name='版记']) | AKN 3.1 无抄送机关元素; 与主送机关同属“收文者”缺口, 只能扩展, 属重点待讨论项; conclusions 里的 container@name='版记' 是合法原生容器, 但 doc 在 attachments 之后没有容器位置, 故统一放 proprietary |
| 主送机关移至版记 (`record.addressee_moved`) | 7.4.2 | 扩展承载 | proprietary/gbt:gbt9704/gbt:record/gbt:addresseeMoved | 主送机关移入版记是版式位置变化, AKN 只记录“哪些机关在版记中” |
| 印发机关和印发日期 (`record.printing`) | 7.4.3 | 语义近似 | meta/publication(@name,@date) + proprietary/gbt:record/gbt:printing | publication 的语义是官方公布事件(公报/公告), 印发是机关内部印制发行; 语义偏移, 显示文本另存扩展 |
| 版记中的其他要素 (`record.other_elements`) | 7.4.3 | 扩展承载 | proprietary/gbt:record + presentation/gbt:record-lines | 版记其他要素无 AKN 语义, 只能结构化扩展 |

### 3.7 页码

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 页码 (`page.number`) | 7.5 | 无对应(版式/印制) | 无 AKN 语义; presentation/gbt:page-number | 页码是 manifestation 的呈现属性, AKN 无页码元素; eop/@number 只能记录原文分页锚点, 单双页位置/一字线/空白页规则属版式 |

### 3.8 横排表格

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 公文中的横排表格 (`table.landscape`) | 8 | 语义近似 | akn:table + presentation(横排/表头方向) | 表格结构可原生表达; 横排纸张、单双页表头在订口/切口属版式, 需渲染层处理 |

### 3.9 计量单位与标点数字

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 计量单位、标点符号和数字用法 (`content.units`) | 9 | 无对应(版式/印制) | 无 AKN 语义; 由内容校验/排版层执行, 规则记录在 presentation | 计量单位/标点/数字用法是文字内容规范, 不是文档结构; AKN 不表达, 应作为校验规则 |

### 3.10 特定格式

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 信函格式 (`special.letter`) | 10.1 | 语义近似 | FRBRWork/FRBRsubtype(信函) + doc + presentation/gbt:format[name=letter] | AKN 无信函根元素/版式; 用通用 doc + FRBRsubtype 表达文种, 版式参数(170 mm 双线/上 30 mm/下 20 mm)进扩展 |
| 命令(令)格式 (`special.order`) | 10.2 | 语义近似 | FRBRWork/FRBRsubtype(命令) + doc + presentation/gbt:format[name=order] | AKN 的 act/bill 等不覆盖中国行政命令(令); 使用通用 doc + FRBRsubtype + 版式扩展 |
| 纪要格式 (`special.minutes`) | 10.3 | 语义近似 | FRBRWork/FRBRsubtype(纪要) + conclusions/p[@gbt:role='attendance'] + presentation | AKN 的 debate/debateReport 面向议会辩论记录, 与中文“纪要”不同; 出席/请假/列席名单无原生元素, 用 conclusions 段落 + 扩展属性 |

### 3.11 式样

| 标准要素 | 条款 | 类别 | AKN 对应/存放位置 | 冲突/说明 |
|---|---|---|---|---|
| 公文式样(图 1~图 12) (`sample.figures`) | 11 | 无对应(版式/印制) | 无 AKN 语义; docs/figures + presentation/gbt:element[sample.figures] | 式样图是标准的说明性附图, 不属于任何具体公文实例 |

## 4. 冲突与待讨论清单

### 4.1 语义近似(AKN 有相近元素, 但语义偏移)

- **密级和保密期限** (7.2.2) → meta/classification/keyword(dictionary=#gbt_dict_secrecy) + preface/p[@gbt:role='secrecy']; 冲突: classification/keyword 的语义是主题分类/受控词表, 不是保密等级; 密级和保密期限被当作关键词承载, 语义偏移
- **紧急程度** (7.2.3) → meta/classification/keyword(dictionary=#gbt_dict_urgency) + preface/p[@gbt:role='urgency']; 冲突: 紧急程度同样不是主题关键词; 若 AKN 未来提供 docUrgency 类元素应迁移
- **发文机关标志** (7.2.4) → FRBRauthor(TLCOrganization) + preface/p[@gbt:role='issuerMark'] + presentation/gbt:font[issuer_mark]; 冲突: 发文机关标志 = 机关名称(语义) + 红色小标宋/35 mm 版头位置(视觉); AKN 只表达机关, 视觉部分只能进扩展
- **联合行文发文机关标志** (7.2.4) → 多个 FRBRauthor(TLCOrganization) + preface/p[@gbt:role='issuerMark'] + presentation; 冲突: 联合行文“主办机关在前、文件二字在右、上下居中”是版式规则; AKN 用多个并列 author 表达机关集合, 不表达排布
- **签发人** (7.2.6) → workflow/step(@date,@by,@as) + preface/p/docProponent + references/TLCPerson,TLC Role; 冲突: AKN 无 docSignatory 元素; 签发人既是签名者(workflow 步骤), 又近似 docProponent(提议人), 语义不完全等价
- **发文机关署名、成文日期和印章(加盖印章)** (7.3.5.1) → conclusions/p/signature + FRBRauthor; 印章图形/位置在扩展; 冲突: AKN 的 signature 是文本签名标识, 不表达红色印章图形、下压位置与“不得空白印章”等版式要求; 印章是图形资源
- **发文机关署名、成文日期(不加盖印章)** (7.3.5.2) → conclusions/p/signature + FRBRauthor; 冲突: 署名与日期的相对位置(右空二字/首字右移二字)是版式, AKN 不表达
- **签发人签名章** (7.3.5.3) → conclusions/p/signature + workflow(签发人) + 签名章占位扩展; 冲突: 签名章是图形资源; 签发人职务、签名章、成文日期的上下居中关系属版式; AKN 只能表达签发人身份与日期
- **附注** (7.3.6) → meta/notes/note(@marker,@placement) + conclusions/p[@gbt:role='annotation']; 冲突: AKN note 的语义是脚注/尾注/页边注; 公文附注是成文日期后的说明(如“此件公开发布”), 语义接近但不完全一致
- **印发机关和印发日期** (7.4.3) → meta/publication(@name,@date) + proprietary/gbt:record/gbt:printing; 冲突: publication 的语义是官方公布事件(公报/公告), 印发是机关内部印制发行; 语义偏移, 显示文本另存扩展
- **公文中的横排表格** (8) → akn:table + presentation(横排/表头方向); 冲突: 表格结构可原生表达; 横排纸张、单双页表头在订口/切口属版式, 需渲染层处理
- **信函格式** (10.1) → FRBRWork/FRBRsubtype(信函) + doc + presentation/gbt:format[name=letter]; 冲突: AKN 无信函根元素/版式; 用通用 doc + FRBRsubtype 表达文种, 版式参数(170 mm 双线/上 30 mm/下 20 mm)进扩展
- **命令(令)格式** (10.2) → FRBRWork/FRBRsubtype(命令) + doc + presentation/gbt:format[name=order]; 冲突: AKN 的 act/bill 等不覆盖中国行政命令(令); 使用通用 doc + FRBRsubtype + 版式扩展
- **纪要格式** (10.3) → FRBRWork/FRBRsubtype(纪要) + conclusions/p[@gbt:role='attendance'] + presentation; 冲突: AKN 的 debate/debateReport 面向议会辩论记录, 与中文“纪要”不同; 出席/请假/列席名单无原生元素, 用 conclusions 段落 + 扩展属性

### 4.2 AKN 无原生元素, 用扩展承载

- **份号** (7.2.1) → preface/p[@gbt:role='copyNumber'] + gbt:digits; 冲突: AKN 无份号元素; 理论上可借用 FRBR Item 层标识同一 expression 的不同副本, 但 AKN 没有 item 级编号元素
- **主送机关** (7.3.2) → preface/p[@gbt:role='addressee'](纯文本); 无 AKN 元素; 冲突: AKN 3.1 没有主送机关/收文者元素(docAddressee 不存在); 只能作为 preface 普通段落 + 扩展属性, 属重点待讨论项
- **附件说明** (7.3.4) → conclusions/p[@gbt:role='attachmentNote'] + attachments/attachment 清单; 冲突: AKN 无“附件说明”元素; 附件说明是正文对附件的引用清单, 用 conclusions 段落 + 扩展属性表达, 附件实体在 attachments
- **附件标注与附件标题** (7.3.7) → attachment/doc/preface/p[@gbt:role='attachmentLabel'\|'attachmentTitle'] + docTitle; 冲突: “附件”二字与顺序号、附件标题在版心第三行是版式; AKN 用嵌套 doc 的 preface 承载说明文字
- **附件与正文不一起装订时的标注** (7.3.7) → attachment/doc/preface/p[@gbt:role='attachmentStandaloneLabel'] + gbt:attachment-standalone; 冲突: AKN 无“附件与正文分开装订时以发文字号标注”的概念
- **抄送机关** (7.4.2) → proprietary/gbt:gbt9704/gbt:record/gbt:cc(备选: conclusions/container[@name='版记']); 冲突: AKN 3.1 无抄送机关元素; 与主送机关同属“收文者”缺口, 只能扩展, 属重点待讨论项; conclusions 里的 container@name='版记' 是合法原生容器, 但 doc 在 attachments 之后没有容器位置, 故统一放 proprietary
- **主送机关移至版记** (7.4.2) → proprietary/gbt:gbt9704/gbt:record/gbt:addresseeMoved; 冲突: 主送机关移入版记是版式位置变化, AKN 只记录“哪些机关在版记中”
- **版记中的其他要素** (7.4.3) → proprietary/gbt:record + presentation/gbt:record-lines; 冲突: 版记其他要素无 AKN 语义, 只能结构化扩展

### 4.3 纯版式/印制/内容规则(不进 AKN 语义)

- **公文用纸技术指标** (4) → 无 AKN 语义; presentation/gbt:element[paper.technical]; 冲突: 纸张物理指标是印后/材料要求, 不属于文档语义; 仅作为呈现参数存档
- **幅面尺寸(A4)** (5.1) → 无 AKN 语义; presentation/gbt:page; 冲突: 幅面是 manifestation 的物理属性; AKN 无页面尺寸元素
- **页边与版心尺寸** (5.2.1) → 无 AKN 语义; presentation/gbt:margins + gbt:text-area; 冲突: 页边/版心属于版面几何; AKN 只描述结构, 不描述版心
- **字体和字号(默认)** (5.2.2) → 无 AKN 语义; presentation/gbt:font[slot=body]; 冲突: 字体字号是呈现属性; AKN 的文本内容不带字体
- **行数和字数** (5.2.3) → 无 AKN 语义; presentation/gbt:text-area + gbt:grid; 冲突: 22 行×28 字是版面网格; AKN 无行数字数概念
- **字、行(术语定义)** (3.1/3.2) → 无 AKN 语义; presentation/gbt:grid(line-unit-pt/char-pitch-pt) + presentation/gbt:element[layout.units]; 冲突: “字/行”是排版距离单位; 术语定义值(16 pt/30 pt)与 22 行×28 字网格值(15.79 pt/28.99 pt)是两套常量, AKN 只能存参数
- **文字的颜色** (5.2.4) → 无 AKN 语义; presentation/gbt:colors; 冲突: 文字颜色是呈现属性
- **制版要求** (6.1) → 无 AKN 语义; presentation/gbt:element[printing.plate]; 冲突: 制版质量要求不属于文档内容
- **印刷要求** (6.2) → 无 AKN 语义; presentation/gbt:colors(黑色/红色油墨); 冲突: 印刷油墨色值(CMY K)属于印后工艺; RGB/CMYK 由阶段 2 处理
- **装订要求** (6.3) → 无 AKN 语义; presentation/gbt:element[printing.binding]; 冲突: 装订方式/订位不属于文档结构
- **版头中的分隔线** (7.2.7) → 无 AKN 语义; presentation/gbt:element[header.red_line]; 冲突: 红色分隔线是版头视觉分隔, AKN 无版头/分隔线概念
- **特殊情况说明(调整行距字距)** (7.3.5.5) → 无 AKN 语义; 由阶段 2 排版算法执行, 参数在 presentation; 冲突: “调整行距、字距”是排版算法, AKN 无法表达
- **版记中的分隔线** (7.4.1) → 无 AKN 语义; presentation/gbt:record-lines; 冲突: 版记分隔线(粗 0.35/细 0.25 mm)是纯版式
- **页码** (7.5) → 无 AKN 语义; presentation/gbt:page-number; 冲突: 页码是 manifestation 的呈现属性, AKN 无页码元素; eop/@number 只能记录原文分页锚点, 单双页位置/一字线/空白页规则属版式
- **计量单位、标点符号和数字用法** (9) → 无 AKN 语义; 由内容校验/排版层执行, 规则记录在 presentation; 冲突: 计量单位/标点/数字用法是文字内容规范, 不是文档结构; AKN 不表达, 应作为校验规则
- **公文式样(图 1~图 12)** (11) → 无 AKN 语义; docs/figures + presentation/gbt:element[sample.figures]; 冲突: 式样图是标准的说明性附图, 不属于任何具体公文实例

## 5. 标准条款粒度覆盖(77 项)

`docs/gbt9704-2012-elements.json` 把标准拆成 77 个细分要素; 下表说明每一项落到哪个映射单元, 因此
“标准里的每一项”都有明确去向(细分项继承所属映射单元的类别与扩展方式)。

| 细分要素 id | 名称/条款 | 归属映射单元 | 类别 |
|---|---|---|---|
| `page.paper` | 公文用纸幅面(5.1) | `paper.a4` | 无对应(版式/印制) |
| `page.type_area` | 版心尺寸(5.2.1) | `layout.margins` | 无对应(版式/印制) |
| `page.margin_top` | 天头 37±1 mm(5.2.1) | `layout.margins` | 无对应(版式/印制) |
| `page.margin_left` | 订口 28±1 mm(5.2.1) | `layout.margins` | 无对应(版式/印制) |
| `page.margin_bottom` | 地脚 35 mm(5.2.1 推导) | `layout.margins` | 无对应(版式/印制) |
| `page.margin_right` | 切口 26 mm(5.2.1 推导) | `layout.margins` | 无对应(版式/印制) |
| `page.line_grid` | 22 行网格(5.2.3) | `layout.grid` | 无对应(版式/印制) |
| `page.char_grid` | 28 字网格(5.2.3) | `layout.grid` | 无对应(版式/印制) |
| `page.font_default` | 默认 3 号仿宋(5.2.2) | `layout.default_font` | 无对应(版式/印制) |
| `page.text_color` | 文字黑色(5.2.4) | `layout.text_color` | 无对应(版式/印制) |
| `unit.char` | 字(3.1) | `layout.units` | 无对应(版式/印制) |
| `unit.line` | 行(3.2) | `layout.units` | 无对应(版式/印制) |
| `print.plate` | 制版(6.1) | `printing.plate` | 无对应(版式/印制) |
| `print.duplex` | 双面印刷(6.2) | `printing.ink` | 无对应(版式/印制) |
| `print.registration` | 页码套正(6.2) | `printing.ink` | 无对应(版式/印制) |
| `print.ink_black` | 黑墨 BL100%(6.2) | `printing.ink` | 无对应(版式/印制) |
| `print.ink_red` | 红墨 Y80%+M80%(6.2) | `printing.ink` | 无对应(版式/印制) |
| `bind.side` | 左侧装订(6.3) | `printing.binding` | 无对应(版式/印制) |
| `bind.pagination_tolerance` | 页码误差 4 mm(6.3) | `printing.binding` | 无对应(版式/印制) |
| `bind.trim_tolerance` | 成品误差±2 mm(6.3) | `printing.binding` | 无对应(版式/印制) |
| `bind.staple_position` | 钉位距上下边缘 70 mm(6.3) | `printing.binding` | 无对应(版式/印制) |
| `bind.staple_riding` | 骑马订(6.3) | `printing.binding` | 无对应(版式/印制) |
| `bind.staple_side` | 平订(6.3) | `printing.binding` | 无对应(版式/印制) |
| `bind.wrapped` | 包本装订(6.3) | `printing.binding` | 无对应(版式/印制) |
| `header.copy_number` | 份号(7.2.1) | `header.copy_number` | 扩展承载 |
| `header.secrecy` | 密级和保密期限(7.2.2) | `header.secrecy` | 语义近似 |
| `header.urgency` | 紧急程度(7.2.3) | `header.urgency` | 语义近似 |
| `header.authority_mark` | 发文机关标志(7.2.4) | `header.issuer_mark` | 语义近似 |
| `header.doc_number` | 发文字号(7.2.5) | `header.doc_number` | 直接对应 |
| `header.signatory` | 签发人(7.2.6) | `header.signers` | 语义近似 |
| `header.separator_line` | 版头红色分隔线(7.2.7) | `header.red_line` | 无对应(版式/印制) |
| `body.title` | 标题(7.3.1) | `body.title` | 直接对应 |
| `body.main_recipient` | 主送机关(7.3.2) | `body.addressee` | 扩展承载 |
| `body.text` | 正文(7.3.3) | `body.text` | 直接对应 |
| `body.hierarchy_level_1` | 一级序数“一、”(7.3.3) | `body.hierarchy` | 直接对应 |
| `body.hierarchy_level_2` | 二级序数“(一)”(7.3.3) | `body.hierarchy` | 直接对应 |
| `body.hierarchy_level_3` | 三级序数“1.”(7.3.3) | `body.hierarchy` | 直接对应 |
| `body.hierarchy_level_4` | 四级序数“(1)”(7.3.3) | `body.hierarchy` | 直接对应 |
| `body.attachment_note` | 附件说明(7.3.4) | `body.attachment_note` | 扩展承载 |
| `body.attachment_note_item` | 附件说明条目(7.3.4) | `body.attachment_note` | 扩展承载 |
| `body.issuing_signature` | 发文机关署名(7.3.5.1/2) | `body.signature_with_seal` | 语义近似 |
| `body.issue_date` | 成文日期(7.3.5.4) | `body.date_digits` | 直接对应 |
| `body.seal` | 印章(7.3.5.1) | `body.signature_with_seal` | 语义近似 |
| `body.signatory_title` | 签发人职务(7.3.5.3) | `body.signature_seal` | 语义近似 |
| `body.signatory_seal` | 签发人签名章(7.3.5.3) | `body.signature_seal` | 语义近似 |
| `body.annotation` | 附注(7.3.6) | `body.annotation` | 语义近似 |
| `body.attachment` | 附件(7.3.7) | `body.attachment` | 直接对应 |
| `body.attachment_label` | “附件”标注(7.3.7) | `body.attachment_header` | 扩展承载 |
| `body.attachment_title` | 附件标题(7.3.7) | `body.attachment_header` | 扩展承载 |
| `record.separator_thick` | 版记粗分隔线(7.4.1) | `record.separator_lines` | 无对应(版式/印制) |
| `record.separator_thin` | 版记细分隔线(7.4.1) | `record.separator_lines` | 无对应(版式/印制) |
| `record.cc` | 抄送机关(7.4.2) | `record.cc` | 扩展承载 |
| `record.main_recipient` | 主送机关移入版记(7.4.2) | `record.addressee_moved` | 扩展承载 |
| `record.printing_authority` | 印发机关(7.4.3) | `record.printing` | 语义近似 |
| `record.printing_date` | 印发日期(7.4.3) | `record.printing` | 语义近似 |
| `page.number` | 页码(7.5) | `page.number` | 无对应(版式/印制) |
| `page.number_dash` | 页码一字线(7.5) | `page.number` | 无对应(版式/印制) |
| `table.landscape.header_side` | 横排表格表头方向(8) | `table.landscape` | 语义近似 |
| `table.landscape.page_number` | 横排表格页码位置(8) | `table.landscape` | 语义近似 |
| `letterhead.authority_mark` | 信函发文机关标志(10.1) | `special.letter` | 语义近似 |
| `letterhead.double_line_top` | 信函上红色双线(10.1) | `special.letter` | 语义近似 |
| `letterhead.double_line_bottom` | 信函下红色双线(10.1) | `special.letter` | 语义近似 |
| `letterhead.header_elements` | 信函份号/密级/紧急程度(10.1) | `special.letter` | 语义近似 |
| `letterhead.doc_number` | 信函发文字号(10.1) | `special.letter` | 语义近似 |
| `letterhead.title` | 信函标题(10.1) | `special.letter` | 语义近似 |
| `letterhead.bottom_line_text` | 下双线上文字(10.1) | `special.letter` | 语义近似 |
| `letterhead.page_number` | 信函首页不编页码(10.1) | `special.letter` | 语义近似 |
| `letterhead.record` | 信函版记(10.1) | `special.letter` | 语义近似 |
| `order.authority_mark` | 命令发文机关标志(10.2) | `special.order` | 语义近似 |
| `order.order_number` | 令号(10.2) | `special.order` | 语义近似 |
| `order.body` | 命令正文(10.2) | `special.order` | 语义近似 |
| `order.signatory` | 命令签发人/签名章/日期(10.2→7.3.5.3) | `special.order` | 语义近似 |
| `minutes.mark` | 纪要标志(10.3) | `special.minutes` | 语义近似 |
| `minutes.attendance` | 出席人员名单(10.3) | `special.minutes` | 语义近似 |
| `minutes.leave_absent` | 请假/列席人员名单(10.3) | `special.minutes` | 语义近似 |
| `minutes.flexible` | 纪要格式可自定(10.3) | `special.minutes` | 语义近似 |
| `ref.standards` | 规范性引用文件(2) | `content.units` | 无对应(版式/印制) |


## 6. 设计约定(实现层)

1. **AKN 是唯一中间状态**: 语义内容写 AKN 原生元素, 版式写 `presentation`, AKN 没有的元数据写 `proprietary`; 不再维护第二份私有格式。
2. **FRBR**: 每份公文生成 Work/Expression/Manifestation 三层 IRI; 发文字号进 `FRBRnumber`, 标题进 `FRBRname`, 文种进 `FRBRsubtype`, 机关进 `FRBRauthor`。
3. **扩展块**: `akn:proprietary` 承载版记(抄送/印发/移入版记的主送机关)等非呈现元数据; `akn:presentation` 承载全部版面/字体/位置/页码/要素规格(含标准每一个要素)。
4. **可见内容与机器元数据并存**: 标题/发文字号/成文日期等同时出现在 `preface`/`conclusions` 的可读内容与 `meta` 的 FRBR/classification 中, 这是 AKN 的既定用法(内容用于阅读, meta 用于机器处理)。
5. **eId 全局唯一**: 包括嵌套附件 `doc` 内的元素; 由生成器统一编号。
6. **未表达即未定义**: 阶段 2 渲染只允许使用 AKN 中已有的信息; 若渲染需要新参数, 必须先扩展 `presentation` 并更新本表。

## 7. 自检结果

- [x] 44 项标准要素全部有映射, 无遗漏、无悬空引用
- [x] 每个非“直接对应”项都写明了冲突/偏移

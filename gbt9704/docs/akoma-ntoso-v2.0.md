# Akoma Ntoso v2.0 / AKN 3.1 事实来源、schema 与 GB/T 9704 能力矩阵

> 作者：调研队友 `akn-spec`（Team Lead: `lead`）
> 时间：2026-11-07
> 任务：`task-2`（Akoma Ntoso 侧事实来源与官方 schema）
> 范围：只回答"AKN 侧能表达什么、怎么表达、表达不了什么"。GB/T 9704 的条款编号与版式细节由 Lead 在映射层维护。

**一句话结论**：
最新标准是 **OASIS Standard「Akoma Ntoso Version 2.0, Part 2: Specifications of AKN 3.1」（2026-10-02 生效）**；
其**命名空间文档**声明命名空间为 `http://docs.oasis-open.org/legaldocml/ns/akn/3.1`，
但**同一 OS 分发包里真正发布的 XSD** 仍然是 `akomantoso30.xsd`、`targetNamespace = .../ns/akn/3.0`，
只是把根元素 `@version` 固定成了 `"3.1"`。这是一个**官方包内部不一致**，不是我们转述错误；
本仓库把两种口径都做成可校验通道（见 §2、§11、§12）。

---

## 1. 版本与状态

### 1.1 两个版本的事实表

| 项 | v1.0（AKN 3.0） | v2.0（Part 2 = AKN 3.1） |
|---|---|---|
| 标准标题 | *Akoma Ntoso Version 1.0*（Part 1 Vocabulary + Part 2 Specifications） | *Akoma Ntoso Version 2.0, Part 2: Specifications of AKN 3.1* |
| OASIS 阶段 | OASIS Standard | CS01 2026-05-08 → Public Review 2026-07-16 → **OASIS Standard，2026-10-02 生效** |
| 命名空间文档日期 | 2018-08-29 | 2026-10-02 |
| **规范声明的命名空间** | `http://docs.oasis-open.org/legaldocml/ns/akn/3.0` | `http://docs.oasis-open.org/legaldocml/ns/akn/3.1` |
| 官方发布的 XSD 文件名 | `akomantoso30.xsd`（239 K，6862 行） | `akomantoso30.xsd`（251 K，5983 行；文件头 `Final release 11/06/2025 - Akoma Ntoso 3.0`） |
| **XSD 的 targetNamespace** | `.../ns/akn/3.0` | `.../ns/akn/3.0`（**注意：不是 3.1**） |
| 根元素 `akomaNtoso/@version` | XSD 中**不存在**该属性 | XSD 中新增：`fixed="3.1"`（可选属性，出现时只能取 3.1） |
| 官方 schema 目录 | `.../akn-core/v1.0/os/part2-specs/schemas/` | `.../akn-core/v2.0/os/part2-specs/schemas/` |
| 分发 ZIP | `.../v1.0/os/part2-specs/*.zip` | `.../v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.zip`（32,847,536 B） |
| 规范 PDF | `.../v1.0/os/part2-specs/akn-core-v1.0-os-part2-specs.pdf` | `.../v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.pdf`（47,752,703 B） |
| **OS ZIP 实际内容**（读中央目录实测） | — | 仅 4 个文件：`akn-core-v2.0-os-part2-specs.pdf`、`schemas/akomantoso30.xsd`、`schemas/akomantoso30.xml`、`schemas/xml.xsd`；**没有 `akomantoso31.xsd`** |
| 命名空间文档 | https://docs.oasis-open.org/legaldocml/akn-core/v1.0/os/akn-core-v1.0-namespace.html | https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/akn-core-v2.0-namespace.html |

补充事实：

- **v2.0 只发布了 Part 2**：`.../v2.0/os/` 目录下只有 `akn-core-v2.0-namespace.html` 和 `part2-specs/`，
  **没有 v2.0 的 Part 1（XML Vocabulary）**。v2.0 目录里也没有 `part1-vocabulary` 子目录（实测 404）。
  词汇表仍以 v1.0 Part 1（2018-08-29）为准 + v2.0 Part 2 的差异描述：
  https://docs.oasis-open.org/legaldocml/akn-core/v1.0/os/part1-vocabulary/akn-core-v1.0-os-part1-vocabulary.html
- **命名空间 URI 可解引用**：`http://docs.oasis-open.org/legaldocml/ns/akn/3.1` 会跳转到
  `https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/akn-core-v2.0-namespace.html`；
  同一 URI 下的 `akomantoso31.xsd` **不存在**（404）。
- CS01 与 OS 两份 `akomantoso30.xsd` **字节级相同**（`cmp` 验证），即 OS 阶段 schema 未再改动；
  CS01 与 OS 的 `akomantoso30.xml`、`xml.xsd` 亦相同。
- **OS ZIP 已做中央目录实测**（2026-11-07，只下载了 ZIP 末尾 256 KB 并用 Python 解析中央目录）：
  32,847,536 B 的包里只有 4 个条目——`akn-core-v2.0-os-part2-specs.pdf`（压缩 32,786,713 B，
  解压 47,752,703 B）、`schemas/akomantoso30.xsd`（压缩 27,598 B，解压 257,230 B）、
  `schemas/akomantoso30.xml`（30,304 → 156,824 B）、`schemas/xml.xsd`（2,405 → 8,836 B）。
  **整个官方 v2.0 OS 分发包中不存在 3.1 命名空间的 XSD。**
- OASIS 公告（TC 讨论区，标题即结论）：
  https://groups.oasis-open.org/discussion/akoma-ntoso-v2-0-part-2-akn-3-1-approved-as-an-oasis-standard-effective-2-oct
- 第三方新闻稿（旁证，非权威）：https://natlawreview.com/press-releases/akoma-ntoso-v20-approved-oasis-standard-machine-readable-legal-and

### 1.2 命名空间口径：官方包内的矛盾（必读）

三条官方事实并存：

1. v2.0 **命名空间文档**（2026-10-02，OASIS Standard）写明 AKN 3.1 的命名空间是
   `http://docs.oasis-open.org/legaldocml/ns/akn/3.1`。
2. v2.0 OS 包 **schemas 目录**（2026-10-02 目录索引）只有 3 个文件：
   `akomantoso30.xsd`、`akomantoso30.xml`、`xml.xsd`；其中 XSD 的 `targetNamespace` 是 `.../ns/akn/3.0`。
3. 同一 XSD 把根元素 `@version` 固定为 `"3.1"`，文件头写 `Final release 11/06/2025 - Akoma Ntoso 3.0`。

**结论**：官方 v2.0 包没有发布"命名空间为 akn/3.1 的 XSD"。要么是 TC 漏改 XSD 的 targetNamespace，
要么是命名空间文档写错；我们无法在这两个官方文件之外判定谁对。工程上按可校验性优先处理：

| 口径 | 实例根元素写法 | 用哪个 schema 校验 | 适用 |
|---|---|---|---|
| **A（推荐，可机器校验）** | `xmlns=".../ns/akn/3.0"` + `version="3.1"` | 官方 `akomantoso30.xsd` | 需要第三方工具/OASIS 官方 XSD 验证；本仓库 Rust 生成器默认建议 |
| **B（按命名空间文档）** | `xmlns=".../ns/akn/3.1"` + `version="3.1"` | 本地派生 `akomantoso31-adapted.xsd` | 需要严格贴合 v2.0 命名空间文档声明；对端若也用官方 XSD 会校验失败 |

> 口径 A 与 B 的**元素/属性集合完全相同**（适配文件仅替换了两处命名空间字符串），
> 所以切换成本极低。**建议 Lead 在 Rust 侧把命名空间 URI 与校验模式做成常量/开关**，
> 由 CLI 参数选择 A/B；文档与输出默认 A，并显式打印所选 schema 与命名空间。

实测证据（本仓库，2026-11-07）：

```text
# 3.1 命名空间实例 vs 官方 3.0 XSD
minimal.akn.xml:12: Schemas validity error : Element '{http://docs.oasis-open.org/legaldocml/ns/akn/3.1}akomaNtoso':
  No matching global declaration available for the validation root.
# 3.0 命名空间实例 vs 本地适配 3.1 XSD
minimal.akn30.xml:12: Schemas validity error : Element '{http://docs.oasis-open.org/legaldocml/ns/akn/3.0}akomaNtoso':
  No matching global declaration available for the validation root.
```

### 1.3 本仓库 vendoring 结果

`gbt9704/schemas/akn31/`：

| 文件 | 来源 | 性质 | 大小 |
|---|---|---|---:|
| `akomantoso30.xsd` | v2.0 OS `part2-specs/schemas/akomantoso30.xsd` | **官方原件**（ns 3.0） | 257,230 B |
| `akomantoso30.xml` | v2.0 OS 同名文件 | 官方原件（schema-driving XML） | 156,824 B |
| `xml.xsd` | v2.0 OS 同名文件 | 官方原件（W3C xml 命名空间） | 8,836 B |
| `akomantoso31-adapted.xsd` | 由 `akomantoso30.xsd` 仅替换两处命名空间 URI 得到 | **本地派生，非 OASIS 交付物** | 257,230 B |
| `PROVENANCE.md` | 本仓库撰写 | 来源、SHA256、CC-BY 4.0 再分发说明、派生方法 | — |

校验链路完全离线：XSD 只 `import` 同目录的 `./xml.xsd`，无网络依赖。

---

## 2. 根元素与文档骨架

### 2.1 `akomaNtoso` 根

```xml
<akomaNtoso xmlns="http://docs.oasis-open.org/legaldocml/ns/akn/3.0"
            version="3.1">
  <!-- 恰好一个文档根元素（act / bill / doc / ...），可选 components -->
</akomaNtoso>
```

- 类型 `akomaNtosoType`：`documentType` 组（13 选 1）+ 可选 `components`。
- `@version`：3.1 新增，`fixed="3.1"`；**可选**，但一旦出现只能是 `3.1`。
- 根元素上**没有** `xmlns:akn` 约定——AKN 3.x 惯例是**默认命名空间**（无前缀），
  子元素不加前缀；本文档为可读性写作 `akn:` 前缀，实际序列化建议用默认命名空间。
- 根元素（`doc` 等）上带 `xs:unique` 约束：整篇内 `@eId`、`@GUID` 必须唯一。

### 2.2 可选的文档根元素（`documentType` 组，13 个）

| 根元素 | 类型 | 典型用途 |
|---|---|---|
| `act` | hierarchicalStructure | 已生效的法律/法规 |
| `bill` | hierarchicalStructure | 立法草案 |
| `judgment` | judgmentStructure | 判决书 |
| `judgmentSummary` | `judgmentStructure` | 判决摘要（**3.1 新增**，与 `judgment` 同构） |
| `officialGazette` | collectionStructure | 公报（一期） |
| `documentCollection` | collectionStructure | 文档合集 |
| `amendmentList` | `collectionStructure` | 修正案清单 |
| `amendment` | amendmentStructure | 单条修正案 |
| `debateReport` | openStructure | 议事记录 |
| `debate` | debateStructure | 辩论/会议记录 |
| `statement` | openStructure | 声明、决议等 |
| `portion` | `portionStructure` | 文档的独立片段 |
| **`doc`** | **openStructure** | **"其他一切文档"**——AKN 不专门建模的文档类型 |

**GB/T 9704 公文建议根元素：`doc`**。
理由：`doc` 的语义就是"不属于 AKN 专门管理的文档类型"，公文（通知/请示/报告/函/纪要……）
正好属于此类；`act`/`bill` 会带来立法的 `preamble`/`body` 强结构暗示，反而别扭。
若目标是"规范性文件"，可用 `act`。

- `doc` 要求 `@name`（`xsd:string`，必填，自由文本，**无枚举**）。建议稳定值如
  `gbt9704-document`；`@contains` 可选（`originalVersion` 默认 / `singleVersion` / `multipleVersions`）。

### 2.3 `doc`（openStructure）内容模型——顺序固定

```text
meta                          (1, 必填)
coverPage                     (0..1)
preface                       (0..1)
preamble                      (0..1)   公文一般不用
mainBody                      (1, 必填)
conclusions                   (0..1)
attachments                   (0..1)
components                    (0..1)
```

### 2.4 `act` / `bill`（hierarchicalStructure）内容模型

与 `doc` 仅有两点差别：用 **`body`** 代替 `mainBody`；`preamble` 在实践上更重要。

```text
meta (1) → coverPage? → preface? → preamble? → body (1) → conclusions? → attachments? → components?
```

### 2.5 `preface` / `conclusions` / `coverPage`

三者内容模型几乎一致（`prefaceopt`（preface）、`basicopt`（coverPage/conclusions））：

```text
(num?, heading?, subheading?)  ← 3.1 起 via basehierarchy
以下可重复、任意顺序：
  blockElements   = blockList | blockContainer | tblock | toc | ul | ol | table | p | foreign | block
  basicContainers/prefaceContainers = longTitle | formula | container
  crossHeading | interstitial                      ← 3.1 起与 block 平级可用
```

要点：
- **`docTitle`/`docNumber`/`docType`/`docDate` 等不是 `preface` 的直接子元素**，它们是 inline 元素，
  必须放在块（最常见 `<p>`）里，例如 `<p><docTitle>……</docTitle></p>`。
  （XSD 的 `inlineElements` 组把它们放进所有 inline 内容模型，因此**技术上任何 `p` 里都能写**；
  但语义上只该出现在 preface/preamble/coverPage，属"schema 不强制、规范约定"的松约束。）
- `conclusions` 可放 `p`、`container`、`longTitle`、`formula`、`blockList`、`tblock`、`table`、`toc`、`crossHeading`、`interstitial` 等；
  `signature` 是 **inline** 元素，必须包在 `p` 里：`<p><signature>××市人民政府</signature></p>`。
- `container` 是"具名块容器"：`<container name="版记">…</container>`，`@name` 必填、自由文本，
  是承载 GB/T 9704"版记"一类自定义分区的**原生合法**容器。

---

## 3. `meta` 全量元素

### 3.1 `meta` 的顺序（严格 sequence）

```text
identification   (1, 必填)
publication      (0..1)
classification   (0..*)
lifecycle        (0..*)
workflow         (0..*)
analysis         (0..*)
temporalData     (0..*)
references       (0..*)     ← 元素定义是 refItems，choice 可重复；语义上通常只用一份
notes            (0..*)
proprietary      (0..*)
presentation     (0..*)
```

`meta` 没有属性。各子容器大多要求 `@source`（指向 `references` 里的
`TLCPerson`/`TLCOrganization` 的 `@eId`，`#id` 形式），`eId` 可选。

### 3.2 `identification` + FRBR 三层

```xml
<identification source="#agent">
  <FRBRWork> … </FRBRWork>
  <FRBRExpression> … </FRBRExpression>
  <FRBRManifestation> … </FRBRManifestation>
  <FRBRItem> … </FRBRItem>          <!-- 可选 -->
</identification>
```

- `identification/@source` **必填**。
- 三层顺序固定：Work → Expression → Manifestation（→ Item?）。
- 每一层先 `coreProperties`，再叠各层专有属性：

| 层 | coreProperties（所有层共有，顺序固定） | 该层追加属性 |
|---|---|---|
| `FRBRWork` | `FRBRthis`(1) `FRBRuri`(1..*) `FRBRalias`(0..*) `FRBRdate`(1..*) `FRBRauthor`(1..*) `componentInfo`(0..1) `preservation`(0..1) | `FRBRcountry`(1) `FRBRsubtype`(0..1) `FRBRnumber`(0..*) `FRBRname`(0..*) `FRBRprescriptive`(0..1) `FRBRauthoritative`(0..1) |
| `FRBRExpression` | 同左 | `FRBRversionNumber`(0..1) `FRBRauthoritative`(0..1) `FRBRmasterExpression`(0..1) `FRBRlanguage`(**1..\***) `FRBRtranslation`(0..*) |
| `FRBRManifestation` | 同左 | `FRBRportion`(0..1) `FRBRformat`(0..1) |
| `FRBRItem` | 只有 coreProperties | — |

各叶子元素的属性（**序列化时必须写对**）：

| 元素 | 必填属性 | 可选属性 | 说明 |
|---|---|---|---|
| `FRBRthis` | `@value` | — | 该层"本片段"的 IRI |
| `FRBRuri` | `@value` | — | 该层整体 IRI，可多条 |
| `FRBRalias` | `@value` **且** `@name` | — | 别名（`@name` 是别名类型名） |
| `FRBRdate` | `@date` **且** `@name` | `@calName` `@verbatim` | `@date` 类型 = `gYear`\|`date`\|`dateTime`；`@name` 说明是什么日期（如 `generation`、`enactment`、`publication`） |
| `FRBRauthor` | `@href`（`#eId` 或 URI） | `@as`（`TLCRole` 引用） | **"发文机关/作者"就在这里** |
| `FRBRcountry` | `@value` | — | ISO 3166-1 alpha-2，如 `cn` |
| `FRBRsubtype` / `FRBRnumber` / `FRBRname` | `@value` | — | Work 级 IRI 组成要素 |
| `FRBRprescriptive` / `FRBRauthoritative` | `@value`（`xsd:boolean`） | — | 是否规范性 / 权威文本 |
| `FRBRlanguage` | `@language`（RFC 4646，如 `zho`） | — | **语言只能写在这里**（Expression 层） |
| `FRBRformat` | `@value` | — | Internet Media Type，如 `application/xml`（Manifestation 层） |
| `FRBRversionNumber` | `@value` | — | 版本号字符串 |
| `FRBRmasterExpression` | `@href`（**必填**；`@eId` 可选） | — | linkType：指向 master expression |
| `FRBRtranslation` | `@href` | `@fromLanguage` `@authoritative` `@pivot` `@by` | 译文来源 |
| `FRBRportion` | — | `@range` | 片段 |
| `componentInfo` | — | — | 内含 `componentData*`（`@name`/`@href`/`@showAs` 必填） |
| `preservation` | — | — | 任意 `##other` 命名空间内容 |

**FRBR IRI 实践**（AKN 命名约定，非 schema 强制）：
`/akn/{country}/{docType}/{number}/{date}/{component}`，
Expression 以 `@` 结尾（如 `.../main@`），Manifestation 带扩展名（如 `.../main.xml`）。

### 3.3 `doc*` 元素：**存在性清单（务必按此做映射）**

**v2.0/AKN 3.1 schema 中实际存在的 `doc*` 元素（共 16 个）**：

| 元素 | 类型/内容 | 属性 | 语义 |
|---|---|---|---|
| `docType` | inline（可混合文本） | — | 文档自称的类型字符串（**自由文本，无枚举**） |
| `docTitle` | inline | — | 标题 |
| `docNumber` | inline | — | 文号/编号 → **发文字号** |
| `docDate` | mixed inline | `@date`(必填) `@calName` `@verbatim` | 成文/签署日期，可多条 |
| `docProponent` | mixed inline | `@as`（roleRef） | 提案人/发起人 |
| `docAuthority` | inline | — | "提交给的机关"（议会法案场景） |
| `docIntroducer` | inline | — | 提出人 |
| `docPurpose` | inline | — | 目的 |
| `docCommittee` | mixed inline | `@value` | 委员会 |
| `docStage` | inline | — | 阶段 |
| `docStatus` | inline | — | 状态 |
| `docJurisdiction` | inline | — | 管辖区域 |
| `shortTitle` | inline | — | 简称 |
| `docketNumber` | inline | — | 案卷号 |
| `legislature` | mixed inline | `@value` | 届次 |
| `session` | mixed inline | `@value` | 会期 |

**schema 中不存在**（0 个全局元素，属性也没有）——这是本次调研最重要的负面结论：

| 任务中提到的 | 实际状态 | 替代方案 |
|---|---|---|
| `docAuthor` | ❌ 不存在（v1.0 也没有） | `FRBRauthor @href`（+ `references/TLCPerson`/`TLCOrganization`） |
| `docIssuer` | ❌ 不存在 | 同上：`FRBRauthor`；渲染"发文机关标志"另需扩展/渲染层 |
| `docAddressee`（主送机关） | ❌ 不存在 | 只能 `<p>` 文本或 proprietary 扩展 |
| `docSignature` | ❌ 不存在 | `signature`（inline，放 `p` 内，`conclusions` 常用）；署名主体用 `references/TLCOrganization` |
| `docLanguage` | ❌ 不存在 | `FRBRExpression/FRBRlanguage/@language` |
| `docSource` | ❌ 不存在 | `FRBRuri` / `identification/@source` / 任意元素上的 `@source` |
| `docFormat` | ❌ 不存在 | `FRBRManifestation/FRBRformat/@value` |

### 3.4 `classification` / `keyword`

```xml
<classification source="#agent">
  <keyword value="公文" dictionary="#dict-eId"/>
  <keyword value="通知" dictionary="#dict-eId" href="#frag"/>
</classification>
```

- `classification`：`keyword`(**1..\***) + `@source`（必填）。
- `keyword`：`@value`(**必填**) + `@dictionary`(**必填**，`#eId` 形式，指向 `references` 里的词典/概念)
  + `@href`(可选) + 3.1 起还继承 `@refersTo`/`@showAs`/`@shortForm`。
- **没有"主题词表/密级"专用结构**：可用 `keyword` 近似，但语义不完全对应。

### 3.5 其它 meta 容器

| 容器 | 内容模型 | 属性 | 备注 |
|---|---|---|---|
| `publication` | 空（属性承载） | `@date`(必填) `@name`(必填) `@number` `@showAs` `@shortForm` | 公报发布事件 |
| `lifecycle` | `eventRef`(1..*) | `@source` | 文档生命周期事件 |
| `eventRef` | 任意 `##other` 内容 | `@date`(必填) `@source`(必填) `@type` `@refersTo` `@originatingExpression` | |
| `workflow` | `step`(1..*) | `@source` | 审批/签发流程 |
| `step` | 任意 `##other` 内容 | `@date`(必填) `@by`(必填，agentRef) `@as` `@outcome` `@refersTo` | **签发人可放这里**（`@by`→TLCPerson） |
| `analysis` | `activeModifications?` `passiveModifications?` `restrictions?` `judicial?` `parliamentary?` `mappings?` `otherReferences*` `definitions?`(3.1 新增) `otherAnalysis*` | `@source` | |
| `temporalData` | `temporalGroup`(1..*) | `@source` | 版本时效 |
| `references` | `docRefs`+`TLCs` 混合（1..*） | `@source` | `TLCConcept/TLCOrganization/TLCPerson/TLCRole/…` 的 eId 供 `#` 引用 |
| `notes` | `note`(1..*) | `@source` | 脚注/尾注定义 |
| `note` | 富内容（subFlowStructure） | `@marker` `@placement` `@placementBase` `@eId` 等 | 与正文 `noteRef` 配合 |

### 3.6 `proprietary` / `presentation`：唯一的官方扩展点

```xml
<proprietary source="#agent">
  <gbt:layout gbt:paper="A4" …/>      <!-- 自定义命名空间元素 -->
</proprietary>
<presentation source="#agent">
  <gbt:cover gbt:redHeaderLine="true"/>
</presentation>
```

- 两者类型都是 `anyOtherType`：

```xml
<xs:choice>
  <xs:any namespace="##other" processContents="lax" minOccurs="0" maxOccurs="unbounded"/>
</xs:choice>
<!-- 外加 core 属性组（任何 ##other 命名空间的属性）、eId/wId/GUID、@href -->
```

- 关键点：
  1. 扩展元素**必须在非 AKN 命名空间**（`##other`），不能塞 AKN 元素；
  2. `processContents="lax"`：**若调用方提供了该命名空间的 schema 就会校验，否则原样接受**；
  3. **绝大多数内容/元数据元素**都通过 `core` 属性组允许 `##other` 命名空间属性
     （`coreopt`/`corereq`/`metaopt` 等都含 `xs:anyAttribute namespace="##other"`），
     所以也可以写 `<p gbt:lineHeight="28pt">`、`<attachment gbt:sealNeeded="true">`；
     但少数容器只有 `@source`（如 `meta` 无属性、`identification`/`classification`/`attachments`
     仅有 `@source`），不能在这些元素上挂扩展属性；
  4. `proprietary` / `presentation` 在 `meta` 中可重复（0..*）。
- 官方推荐用法：`proprietary` 放非 AKN 元数据，`presentation` 放渲染提示；
  对需要严格校验的扩展，可另外提供自定义 XSD，让 `lax` 生效。

---

## 4. 结构层级（正文）

### 4.1 入口

- `doc` → `mainBody`（`maincontent`）：`componentRef | hierElements | blockElements | containerElements`，1..*；
  3.1 起 `maincontent` 继承 `basehierarchy`，因此 `mainBody` 开头允许 `num? heading? subheading?`。
- `act`/`bill` → `body`（`bodyType`）：`componentRef | hierElements`，1..*；**没有** `num/heading/subheading` 前缀
  （`bodyType` 不继承 `basehierarchy`）。

### 4.2 `hierElements` = 命名层级元素（`ANhier`）或 `hcontainer`

`ANhier` 全集（3.1）：

```text
book, title, chapter, section, article, part, tome, division, clause, rule, subrule,
paragraph, subparagraph, subsection, subpart, subchapter, subtitle, subdivision,
subclause, sublist, level, point, indent, alinea, list, proviso,
crossHeading, interstitial,
schedule, subschedule, mediaContainer, regulation, noteSection, annex, appendix, transitional
```

其中 **3.1 相对 3.0 新增的层级元素**：`schedule`、`subschedule`、`mediaContainer`、
`regulation`、`noteSection`、`annex`、`appendix`；并把 `crossHeading`、`interstitial`
提到层级位置可直接使用（详见 §8）。

### 4.3 命名层级元素的内容模型（`hierarchy`）

所有命名层级元素（`article`/`section`/`paragraph`/…）与 `hcontainer` 共用 `hierarchy`：

```text
(num?, heading?, subheading?)                    ← basehierarchy，各至多 1 次，固定顺序
然后二选一：
  A) content                                     ← 最终文本块
  B) intro?, (componentRef | hierElements)*, wrapUp?
```

- `content`、`intro`、`wrapUp` 都是 `blocksreq`：0..* 个 block 元素（`p`/`table`/`blockList`/…）。
- 属性：`@eId`、`@wId`、`@GUID`、`@refersTo`、`@class/@style/@title`、`@xml:lang/@xml:space/@xml:id`、`@status`、`@period`、`@alternativeTo`、`@defines`…（`corereq`，其中 `@eId` 在 XSD 里是可选）。
  **`@eId` 在 schema 里是可选的**（文档说"使用 AKN 命名约定时必填"），但实务上强烈建议生成。
- **嵌套没有深度/顺序限制**：`hierarchy` 的 B 分支允许 `hierElements` 无限递归，
  即 `article` 里可以放 `section`，`section` 里可以放 `paragraph`……也可以任意混用；
  "article 只能放 paragraph"这类层级约束是**规范约定**（Part 1），XSD **不强制**。

### 4.4 `hcontainer`：自定义层级（GB/T 9704 的关键工具）

```xml
<hcontainer name="level-1">          <!-- @name 必填、xsd:string、无枚举 -->
  <num>一、</num>
  <heading>总体要求</heading>
  <content><p>……</p></content>
</hcontainer>
```

- `hcontainer` 类型 = `hierarchy` + `@name`（必填）。
- `@name` **不是枚举**，是自由文本；惯例用本地层级名（如 `part`、`level-1`、`sec_1__para_2` 之类）。
- 用途：GB/T 9704 正文的"一、（一）1.（1）"层级，用命名层级元素（`paragraph`/`subparagraph`）
  或 `hcontainer` 都能表达；**推荐 `hcontainer + @name`**，语义自解释且不占用 AKN 保留名。

### 4.5 块元素与其它

| 元素 | 类型 | 说明 |
|---|---|---|
| `p` | inline（mixed） | 段落；可含所有 inline（含 `docTitle` 等，见 §2.5 的松约束） |
| `blockContainer` | 3.1 起 = `blockContainerType` + `@name` 可选 | 块的具名分组 |
| `blockList` | `listIntroduction? item+ listWrapUp?` | 列表（语义化） |
| `list` / `sublist` | `hierarchy` | 列表也可作为层级元素 |
| `item` | `basehierarchy? + (blockElements|crossHeading|interstitial)+` | `blockList` 的成员 |
| `ul` / `ol` / `li` | HTML 列表 | 简单列表 |
| `table` | 见 §4.6 | HTML 风格表格 |
| `tblock` | `itemType` + `@name` 可选 | "带标题的块"，用于无编号的带标题小节 |
| `toc` | `tocItem`(1..*) | 目录（3.1 增强，见 §8） |
| `foreign` | `anyOtherType` | 块级扩展容器（数学公式等） |
| `block` | `anyOtherType` | 通用块扩展 |
| `div` | container | HTML 风格容器 |
| `container` | `@name` 必填 + 块内容 | 通用块容器；**preface/conclusions 里也能用** |
| `crossHeading` / `interstitial` | inline / blocksreq | 与层级元素并排的标题 / 层级间的过渡文字 |
| `eop` / `eol` | `markeropt` + `@number`/`@breakAt`/`@breakWith` | **分页 / 换行标记** |
| `marker` | `markerreq` + `@name` 必填 | 通用标记（自定义语义） |
| `authorialNote` | subFlowStructure + notes 属性 | 作者批注 |

### 4.6 表格支持（3.1 增强）

```xml
<table border="1">
  <caption>…</caption>            <!-- 3.1 起是块级内容（blocksopt） -->
  <colgroup span="3"><col span="1"/></colgroup>   <!-- 3.1 新增 -->
  <thead><tr><th>…</th></tr></thead>              <!-- 3.1 新增 -->
  <tbody><tr><td>…</td><td>…</td></tr></tbody>    <!-- 3.1 新增 -->
  <tfoot>…</tfoot>                                <!-- 3.1 新增 -->
</table>
```

- `table`：`@width @border @cellspacing @cellpadding`；`tr` 至少 1 个。
- `td` / `th`：3.1 起基类改为 `subFlowStructure`（**可放块/层级/嵌套表**），并通过
  `@rowspan`/`@colspan` 控制合并。
- `caption` 3.1 起从 inline 变为 `blocksopt`（块级标题）。

### 4.7 `num` / `heading` / `subheading` / `intro` / `content` / `wrapUp`

| 元素 | 类型 | 位置 | 语义 |
|---|---|---|---|
| `num` | inline | basehierarchy，至多 1，第一位 | 编号（"一、"/"（一）"/"1."） |
| `heading` | inlinereq | 第二位 | 标题 |
| `subheading` | inlinereq | 第三位 | 副标题 |
| `intro` | blocksreq | hierarchy B 分支，最前 | 引导下面各子层级的段落 |
| `content` | blocksreq | hierarchy A 分支 | 最终正文块 |
| `wrapUp` | blocksreq | hierarchy B 分支，最后 | 收束段 |

> `num`/`heading`/`subheading` 只在**这个固定顺序、各至多一次**的意义上受限；
> 它们都是 inline，可以再包含 `ins`/`del`/`ref` 等。

---

## 5. `docType` 与"文档类型"判定

- `docType` 元素是 **inline 自由文本**：`<p><docType>通知</docType></p>`。
  **XSD 中没有任何枚举/pattern 约束**；v1.0 与 v2.0 都是如此。
- 机器可判定的"文档类型"来自三处：
  1. **根元素**（`doc`/`act`/`bill`/`judgment`/…，13 选 1，XSD 强制）；
  2. **`FRBRsubtype/@value`**（Work 层，自由字符串，进入 Work IRI）；
  3. `docType` 文本（仅人类可读）。
- 因此不存在"docType 枚举值全集"这种东西；若 Lead 需要稳定的公文种类机读编码，
  建议：根元素 `doc` + `FRBRsubtype value="notice|request|report|letter|minutes|order|…"` +
  `docType` 写中文名。**不要用 `docType` 文本做程序判断**。

---

## 6. 附件与 `conclusions`

### 6.1 `attachments` / `attachment`

- `attachments` → `attachment`(**1..\***)。
- `attachment` 类型 `docContainerType`，**内容恰好一个子元素**，五选一：

```text
documentType   ← 内嵌一份完整 AKN 文档（doc/act/…）
crossHeading   ← 3.1 起可用
interstitial
toc
documentRef    ← 引用外部文档（linkType，@href 必填；@eId 可选）
```

- `attachment` 自身属性 = `corereq`（`@eId`/`@wId`/`@GUID`、`@refersTo`、`@class/@style/@title`、
  `@status`、`@period`、`@alternativeTo`、`@defines`、`@xml:*`、任意 `##other` 属性）。
  **没有 `@href` / `@src`** —— 想引用外部文件要用子元素 `<documentRef href="…"/>`。
- 因此"附件说明"（附件标题列表）与"附件实体"是两件事：
  - 附件实体：`<attachment><doc name="annex-1">…</doc></attachment>`（内嵌）或
    `<attachment eId="att-2"><documentRef eId="ref-2" href="./annex-2.xml"/></attachment>`（外链）；
  - "附件：1. 标题"这类说明文字：AKN 无专用元素，放 `conclusions`/`mainBody` 的 `p`，
    或 `proprietary` 扩展（见 §9 能力矩阵）。
- 多附件 → 多个 `attachment`，各自内嵌/外链。

### 6.2 `conclusions` 能放什么

`basicopt`：`(num?, heading?, subheading?)` + 可重复的
`blockElements`（`p`/`table`/`blockList`/`blockContainer`/`tblock`/`toc`/`ul`/`ol`/`foreign`/`block`）、
`basicContainers`（`longTitle`/`formula`/`container`）、`crossHeading`、`interstitial`。

- **`signature` 是 inline**，只能出现在块里：`<p><signature>××市人民政府</signature></p>`。
- "版记"（抄送机关、印发机关、印发日期）没有原生结构：可用
  `<container name="版记"><p>…</p></container>`（合法，但语义靠 `@name` 约定），
  或用 `proprietary` 扩展（推荐，见 §9）。

---

## 7. 3.0 → 3.1 差异（基于两份**官方 XSD** 的实测 diff）

方法：取官方 **v1.0 OS（2018-08-29）`akomantoso30.xsd`**（6862 行）与
**v2.0 OS（2026-10-02）`akomantoso30.xsd`**（5983 行），解析 `xsd:schema` 下的全局声明，
**忽略 `xsd:annotation` 文档差异**后做结构比对。
复现脚本与两份基线文件：`gbt9704/.scratch/akn-diff/`（`diff_schema.py`，基线 SHA256：
v1.0 = `6f61fe84…c4fd`，v2.0 = `8f1fe3e1…b70a`）。
另：v1.0 OS 的 XSD 在本仓库**未**作为交付物 vendoring，只在 `.scratch/` 作对比基线。
注意：两份 XSD 都叫 `akomantoso30.xsd`，targetNamespace 都是 `.../ns/akn/3.0`；
v1.0 是 2018 年 OS，v2.0 包内是 `Final release 11/06/2025` 版，即 **AKN 3.1 的实际 schema 内容**。

### 7.1 新增的全局元素（26 个）

| 类别 | 新增元素 |
|---|---|
| 新增根元素 | `judgmentSummary` |
| 层级元素（`ANhier`） | `schedule`、`subschedule`、`mediaContainer`、`regulation`、`noteSection`、`annex`、`appendix` |
| 司法/法院 | `court`（`maincontent` 类型）、`witness`（inline，`ANheaderInline`） |
| 定义体系（`analysis`） | `definitions`、`definition`、`definitionHead`、`definitionBody`、`defBody`、`near`、`scope` |
| 目录（TOC） | `tocHeading`、`tocNum`、`tocPointer` |
| 表格 | `colgroup`、`thead`、`tbody`、`tfoot`、`col` |
| 扩展 inline | `foreignText` |

**没有**新增 `docIssuer`、`docAddressee`、`docAuthor`、`docSignature`、`docLanguage`、`docSource`、`docFormat`。
（这些在 v1.0 与 v2.0 中都不存在；`doc*` 家族仍只有 §3.3 列出的 16 个。）

### 7.2 新增类型 / 属性组 / 简单类型

- complexType：`definitionType`、`headeropt`、`inlineToc`
- group：`inlineTocCM`、`inlineTocElements`
- attributeGroup：`defines`、`nameopt`、`tocPointerType`、`unit`
- simpleType：`dateType`、`definesRef`、`recordedTimeType`、`tocPointerType`

### 7.3 结构/语义改动（只列**真实**改动；仅"显式写出 minOccurs=1"的不计）

| 位置 | 3.0（v1.0 OS） | 3.1（v2.0 OS） |
|---|---|---|
| `akomaNtoso` | 无 `@version` | 新增 `@version`，`fixed="3.1"` |
| `documentType` 组 | 12 个根元素 | + `judgmentSummary` |
| `ANhier` 组 | — | + `crossHeading`、`interstitial`、`schedule`、`subschedule`、`mediaContainer`、`regulation`、`noteSection`、`annex`、`appendix` |
| `ANheaderInline` | — | + `witness` |
| `ANinline` | — | + `foreignText` |
| `ANsemanticInline` | — | + `defBody` |
| `header` 元素 | `blocksopt` | `headeropt`（新增 `court` 分支） |
| `table` | `caption? tr+` | `caption? colgroup* thead? (tbody* | tr+) tfoot?` |
| `caption` | inline | `blocksopt`（块级） |
| `td` / `th` | `blocksopt` | `subFlowStructure`（可放块/层级/嵌套表） |
| `tocItem` | `inline` | `inlineToc`（+ `tocNum`/`tocHeading`/`tocPointer`） |
| `toc` | `tocItem` 至少 1（隐式） | 显式 `tocItem(1..*)` |
| `maincontent` / `prefaceopt` / `basicopt` | 纯 choice | 改为 `basehierarchy + choice`，可带 `num/heading/subheading`，preface/conclusions 还可带 `crossHeading`/`interstitial` |
| `docContainerType`（attachment 等） | — | + `crossHeading` |
| `blockContainer` / `tblock` | 无 `@name` | + 可选 `@name` |
| `def` / `term` | — | + `@quote` |
| `quantity` | — | + `@unit` |
| `near` | — | + `@modLevel` |
| `analysis` | 无 `definitions` | + `definitions?` |
| `metaopt`（大多数 meta 元素基类） | 无 `@refersTo`/`@showAs`/`@shortForm` | + `refers` + `showopt`（属性上移） |
| `valueType` / `publication` / `keyword` / `FRBRportion` | 各自显式声明 `refers`/`showopt` | 显式声明被移除，但它们的基类都是 `metaopt`，因此**属性能力不变**（纯重构） |
| `date` 属性组 | `@date` = union(`date`,`dateTime`) | 改用 `dateType` = union(`gYear`,`date`,`dateTime`)；+ `@calName` `@verbatim` |
| `timeType` | 枚举 `startEvent`/`endEvent` | 改为 union(`xsd:time`,`xsd:dateTime`)；原枚举迁到新 `recordedTimeType`（`@time` 与 `recordedTime/@type` 的实际取值能力不变，纯重构） |
| `statusType` | 无 `informative` | + `informative` |
| `posType` | — | + `between`、`fromTo` |
| `remarkType` | — | + `source` |
| `level` 属性组（tocItem 等） | `@level` 必填 | `@level` 可选 |
| `recordedTime` | `@type` = `timeType` | `@type` = `recordedTimeType`（枚举 `startEvent`/`endEvent`） |
| `definitions` 体系 | 无 | 新增 `definitions`/`definition`（`definitionHead(1..*)`+`definitionBody(1..*)`+`condition*`+`scope*`） |
| `unique` 约束 | 各根元素 `@eId`/`@GUID` 唯一 | 不变 |

### 7.4 兼容性判断

- **向后兼容（3.0 实例大多可被 3.1 schema 校验）**：没有删除元素/类型；绝大多数差异是"放宽"
  （新增可选元素/属性、表格与 TOC 增强、inline 组扩大、`@level` 由必填改可选）。
- **唯一的真实收紧点**：`caption` 由 `inline`（混合文本）改为 `blocksopt`（块序列，非 mixed）。
  3.0 里合法的 `<caption>表 1</caption>` 在 3.1 需要写成 `<caption><p>表 1</p></caption>`。
  这对"表格标题"生成逻辑有直接影响，**Rust 生成器必须按 3.1 的块级 caption 生成**。
- 其余"属性组搬家"（`valueType`/`publication`/`keyword`/`FRBRportion` 的 `@refersTo`/`@showAs`/`@shortForm`，
  `timeType` 与 `recordedTimeType`）都是**纯类型重构**，实际可用属性/取值不变，不是破坏性变更。
- **对 GB/T 9704 项目的意义**：3.1 增强的 `table`（thead/tbody/tfoot/colgroup）、
  `toc`（带页码的 tocPointer）、`definitions`、`@version` 都与公文场景相关；
  但 `docIssuer/docAddressee` 等仍**没有**原生元素——所以"发文机关/主送机关/抄送机关"
  无论 3.0 还是 3.1 都需要 mapping 层或扩展。

> ⚠️ 我们**没有**拿到 v2.0 Part 2 规范的正文（47 MB PDF、32 MB ZIP 在本次网络条件下多次
> 下载失败/被重置）。上面的 3.1 差异完全来自**官方 XSD 的机器比对**，是"schema 事实"；
> 规范正文里可能还有"推荐用法/示例"层面的差异，未覆盖。规范 URL 已列入 §13，可后续补齐。

---

## 8. 能力矩阵：GB/T 9704 公文要素 → AKN

判定口径：
- **Direct**：AKN 原生元素/属性可直接承载，语义基本吻合；
- **Extension**：需 `proprietary`/`presentation` + 自定义命名空间，或"原生元素 + 约定"才能承载；
- **No-fit**：AKN 无对应概念，必须由渲染层（Rust 排版器）解决。

> 路径写法中 `@` 表示属性；`…` 表示上一层的上下文。命名空间默认 AKN。

| # | GB/T 9704 要素 | AKN 路径 / 写法 | 判定 | 注意 |
|---|---|---|---|---|
| 1 | 发文字号 | `<p><docNumber>×政发〔2026〕1号</docNumber></p>`（preface） | **Direct** | 也可拆 `FRBRWork/FRBRnumber/@value` 做机读；`docNumber` 是自由文本 |
| 2 | 标题 | `<p><docTitle>关于××的通知</docTitle></p>`（preface） | **Direct** | 标题也可作 `FRBRWork/FRBRname/@value`；换行/回行属渲染层 |
| 3 | 成文日期 | `<p><docDate date="2026-11-07">2026年11月7日</docDate></p>`；同时 `FRBRdate`（三层各 1+，`@name="generation"/"enactment"`） | **Direct** | `@date` 必填；中文日期写文本节点，规范化值写 `@date` |
| 4 | 发文机关 | `FRBRauthor/@href` → `references/TLCOrganization`；版面上可 `<p><docAuthority>××市人民政府</docAuthority></p>` | **Extension**（版头绘制） | 无 `docIssuer`；`docAuthority` 语义是"提交给的机关"，不完全等于发文机关，仅可近似 |
| 5 | 签发人 | `<p><signature>×××</signature></p>`（conclusions）；或 `workflow/step/@by` → `TLCPerson` + `@as` | **Extension** | 无 `docSignature`；"签发人："标签与位置属渲染层/`proprietary` |
| 6 | 主送机关 | 无原生元素；`<p>各有关单位：</p>`（mainBody 首段）或自定义 `gbt:addressee` | **Extension** | 无 `docAddressee`；建议 `proprietary` 存结构化值 + 正文 `p` 存可见文本 |
| 7 | 抄送机关 | 无原生元素；`conclusions` 里 `<container name="版记"><p>抄送：…</p></container>` 或 `gbt:cc` | **Extension** | 可用 `keyword` 近似检索，但语义不符 |
| 8 | 版记：印发机关 / 印发日期 | 同上，`conclusions`/`container name="版记"` 或 `gbt:printer`/`gbt:printDate` | **Extension** | `publication/@date`/`@name` 是"公报发布事件"，不是印发，别混用 |
| 9 | 份号 | 无；`proprietary/gbt:copyNumber` | **No-fit → Extension** | AKN 无"份号"概念；只能扩展 |
| 10 | 密级 | 近似：`classification/keyword`（`@dictionary` 指向自定义"密级"词典）；严格：`gbt:secrecy` | **Extension** | 无保密等级元素；`keyword` 是检索用途，不是安全标记 |
| 11 | 紧急程度 | 无；`gbt:urgency` | **Extension** | 可借用 `docStatus`，但语义是"状态"不是"紧急程度" |
| 12 | 印章 | 无；`proprietary/gbt:seal` + 渲染层 | **No-fit** | AKN 不描述图章；`signature` 只能表达署名文字 |
| 13 | 页码 | `eop/@number`（分页点+页码，正文内）；`tocPointer @type="page"`（目录页码） | **Extension/No-fit** | `eop` 是"原文分页标记"，位置由渲染层决定；页脚"— 1 —"样式属排版 |
| 14 | 版心 / 字体 / 字号 / 行距 / 位置 | `presentation` 或 `proprietary` 下 `gbt:*`（如 `gbt:pageSize`、`gbt:marginTopMm`）；或任意元素上的 `gbt:` 属性 | **Extension** | AKN 明确"不负责版式"；这些值对生成器是**输入参数**，不是文档语义 |
| 15 | 附件说明（"附件：1. ××"） | 无专用元素；`conclusions` 的 `p`，或 `gbt:attachmentList` | **Extension** | 与附件实体分开表达 |
| 16 | 附件实体 | `<attachments><attachment eId="att-1"><doc name="annex-1">…</doc></attachment></attachments>` | **Direct** | 内嵌整份 AKN 文档合法；外链用 `<attachment><documentRef href="…"/></attachment>` |
| 17 | 结构层次序数（一、/（一）/1./（1）） | `hcontainer @name="level-1"` + `<num>一、</num>` + `<heading>` + `<content><p>`；或 `article`/`paragraph`/`subparagraph` 递归 | **Direct** | 层级嵌套 XSD 不限制；深度与编号规则属约定；`num` 是 inline |
| 18 | 横排表格 | `<table><thead>…<tbody>…` + `td/@colspan` `@rowspan` | **Direct**（3.1 增强） | 3.0 只有 `caption? tr+`；3.1 才有 thead/tbody/tfoot/colgroup |
| 19 | 信函 / 命令 / 纪要等特殊格式 | 根元素 `doc` + `FRBRsubtype` 区分；版式差异 `presentation`/`proprietary`；结构差异用 `hcontainer`/`container`/`p` | **Extension（结构）+ No-fit（版式）** | 纪要的"会议纪要"表格式版头通常只能渲染层解决 |
| 20 | 主题词 | `classification/keyword`（`@dictionary` 必填） | **Direct/近似** | 领域（`@dictionary`）需建 `TLCConcept` 词典 |
| 21 | 附注 / 联系人 | `p` 文本；脚注用 `meta/notes/note` + 正文 `noteRef`；或 `authorialNote` | **Direct/Extension** | `note` 需与 `noteRef` 配合 |
| 22 | 引用其他公文 | `<ref href="#doc-eId">`；`references/docRefs` | **Direct** | 需维护 `references` |
| 23 | 目录 | `<toc><tocItem level="1"><tocNum>一、</tocNum><tocHeading>…</tocHeading><tocPointer type="page" href="#eop-1">3</tocPointer></tocItem>…` | **Direct**（3.1 增强） | `tocPointer @type="page"` 的 `@href` 指向 `eop` 的 eId |
| 24 | 红头分隔线 / 版头装饰 | 无；`presentation/gbt:*` | **No-fit** | 纯渲染 |
| 25 | 双面印刷 / 骑缝章 | 无 | **No-fit** | 渲染/后处理 |

**给 Lead 的映射建议**（按优先级）：
1. 语义稳定的是 `FRBR*` + `docNumber/docTitle/docDate/signature`，优先用原生；
2. "版头/版记/份号/密级/紧急程度"统一进 `meta/proprietary` 的 `gbt:*`（结构化、可校验、可回归），
   正文可见文本仍放 `p`，二者用 `eId`/约定关联；
3. **版式参数（版心、字体、字号、行距、页码样式）不要进 AKN 正文**，用 `presentation` 或独立
   配置（Rust 侧 `LayoutSpec`），AKN 只存"语义 + 少量定位锚点（`eop`）"。

---

## 9. 可直接抄用的合法骨架

### 9.1 AKN 3.1（命名空间 3.1，口径 B）

```xml
<?xml version="1.0" encoding="UTF-8"?>
<akomaNtoso xmlns="http://docs.oasis-open.org/legaldocml/ns/akn/3.1"
            xmlns:gbt="urn:gbt9704:ext:1.0"
            version="3.1">
  <doc name="gbt9704-document">
    <meta>
      <identification source="#gbt9704">
        <FRBRWork>
          <FRBRthis value="/akn/cn/doc/gbt9704/2026-11-07/main"/>
          <FRBRuri value="/akn/cn/doc/gbt9704/2026-11-07"/>
          <FRBRalias name="officialName" value="示例公文"/>
          <FRBRdate date="2026-11-07" name="generation"/>
          <FRBRauthor href="#gbt9704"/>
          <FRBRcountry value="cn"/>
          <FRBRnumber value="×政发〔2026〕1号"/>
          <FRBRname value="关于×××的通知"/>
          <FRBRprescriptive value="true"/>
        </FRBRWork>
        <FRBRExpression>
          <FRBRthis value="/akn/cn/doc/gbt9704/2026-11-07/main@"/>
          <FRBRuri value="/akn/cn/doc/gbt9704/2026-11-07/main@"/>
          <FRBRdate date="2026-11-07" name="generation"/>
          <FRBRauthor href="#gbt9704"/>
          <FRBRlanguage language="zho"/>
        </FRBRExpression>
        <FRBRManifestation>
          <FRBRthis value="/akn/cn/doc/gbt9704/2026-11-07/main.xml"/>
          <FRBRuri value="/akn/cn/doc/gbt9704/2026-11-07/main.xml"/>
          <FRBRdate date="2026-11-07" name="generation"/>
          <FRBRauthor href="#gbt9704"/>
          <FRBRformat value="application/xml"/>
        </FRBRManifestation>
      </identification>
      <classification source="#gbt9704">
        <keyword value="公文" dictionary="#gbt9704-dictionary"/>
      </classification>
      <proprietary source="#gbt9704">
        <gbt:layout gbt:paper="A4" gbt:marginTopMm="37" gbt:marginBottomMm="35"
                    gbt:marginLeftMm="28" gbt:marginRightMm="26"/>
      </proprietary>
      <presentation source="#gbt9704">
        <gbt:cover gbt:redHeaderLine="true"/>
      </presentation>
    </meta>

    <preface>
      <p><docType>通知</docType></p>
      <p><docTitle>关于×××的通知</docTitle></p>
      <p><docNumber>×政发〔2026〕1号</docNumber></p>
      <p><docDate date="2026-11-07">2026年11月7日</docDate></p>
      <p><docJurisdiction>中华人民共和国</docJurisdiction></p>
    </preface>

    <mainBody>
      <p>各有关单位：</p>
      <hcontainer name="level-1">
        <num>一、</num>
        <heading>总体要求</heading>
        <content><p>……</p></content>
      </hcontainer>
      <hcontainer name="level-1">
        <num>二、</num>
        <heading>组织实施</heading>
        <content><p>……</p></content>
      </hcontainer>
    </mainBody>

    <conclusions>
      <p><signature>××市人民政府</signature></p>
      <p><docDate date="2026-11-07">2026年11月7日</docDate></p>
      <container name="版记">
        <p>抄送：×××。</p>
      </container>
    </conclusions>

    <attachments>
      <attachment eId="att-1">
        <doc name="annex-1"><meta>…</meta><mainBody><p>附件正文。</p></mainBody></doc>
      </attachment>
    </attachments>
  </doc>
</akomaNtoso>
```

> 真实可校验文件见 `gbt9704/examples/akn/minimal.akn.xml`（3.1 命名空间）与
> `minimal.akn30.xml`（3.0 命名空间 + `version="3.1"`，用官方 XSD 校验）。

### 9.2 口径 A（3.0 命名空间 + `version="3.1"`）

把上面 XML 的第一行命名空间改为 `.../ns/akn/3.0`、`version="3.1"` 保留即可；
元素集合与结构**完全不变**。

### 9.3 最小 FRBR 模板（只保留必填项）

```xml
<identification source="#agent">
  <FRBRWork>
    <FRBRthis value="/akn/{cc}/{type}/{num}/{yyyy-mm-dd}/main"/>
    <FRBRuri value="/akn/{cc}/{type}/{num}/{yyyy-mm-dd}"/>
    <FRBRdate date="{yyyy-mm-dd}" name="generation"/>
    <FRBRauthor href="#agent"/>
    <FRBRcountry value="{cc}"/>
  </FRBRWork>
  <FRBRExpression>
    <FRBRthis value="/akn/{cc}/{type}/{num}/{yyyy-mm-dd}/main@"/>
    <FRBRuri value="/akn/{cc}/{type}/{num}/{yyyy-mm-dd}/main@"/>
    <FRBRdate date="{yyyy-mm-dd}" name="generation"/>
    <FRBRauthor href="#agent"/>
    <FRBRlanguage language="zho"/>
  </FRBRExpression>
  <FRBRManifestation>
    <FRBRthis value="/akn/{cc}/{type}/{num}/{yyyy-mm-dd}/main.xml"/>
    <FRBRuri value="/akn/{cc}/{type}/{num}/{yyyy-mm-dd}/main.xml"/>
    <FRBRdate date="{yyyy-mm-dd}" name="generation"/>
    <FRBRauthor href="#agent"/>
  </FRBRManifestation>
</identification>
```

---

## 10. 命名约定与生成器实现要点（Rust 侧）

1. **默认命名空间**：AKN 惯例是不用前缀；Rust 侧用 `quick-xml`/`xml-rs` 时注意
   `elementFormDefault="qualified"` → 所有元素都在 AKN 命名空间。
2. **顺序敏感**：`meta` 子元素、`FRBR*` 子元素、`basehierarchy`（num→heading→subheading）、
   `table` 子元素、`identification`（Work→Expression→Manifestation→Item）都是 **sequence**，
   序列化顺序错就校验失败。
3. **必填属性**：`identification/@source`、`FRBR*/@value`、`FRBRalias/@name`、`FRBRdate/@date`+`@name`、
   `FRBRauthor/@href`、`FRBRlanguage/@language`、`docDate/@date`、`keyword/@value`+`@dictionary`、
   `doc/@name`、`attachment` 恰一个子元素、`eop` 无必填。
4. **可重复**：`FRBRuri`、`FRBRdate`、`FRBRauthor`、`FRBRnumber`、`FRBRname`、`FRBRlanguage`、
   `classification`、`proprietary`、`presentation`、`attachments/attachment`、`temporalGroup` 等。
5. **唯一性**：`@eId`、`@GUID` 全文唯一（XSD `xs:unique`）——生成器必须维护计数器/注册表。
6. **文本 vs 属性**：`docTitle/docNumber/docType/...` 是**文本**（可混合 inline），不是属性；
   `FRBR*` 是**空元素 + `@value`**。
7. **扩展**：所有 `gbt:*` 元素/属性都必须在自定义命名空间；`proprietary`/`presentation`
   允许 `xs:any ##other lax`；也可在任意 AKN 元素上加 `gbt:*` 属性（`core` 的 anyAttribute）。
8. **校验**：生成后立刻 `xmllint --schema`（或 Rust 侧 `libxml2` 绑定）做 CI 门禁。

---

## 11. 校验方法（可复现）

schema 目录：`gbt9704/schemas/akn31/`（离线，只依赖同目录 `xml.xsd`）。

```sh
# 1) 3.1 命名空间实例 → 本地适配 schema
cd gbt9704/examples/akn
xmllint --noout --schema ../../schemas/akn31/akomantoso31-adapted.xsd minimal.akn.xml
# → minimal.akn.xml validates

# 2) 3.0 命名空间实例（version="3.1"）→ 官方 OS schema
xmllint --noout --schema ../../schemas/akn31/akomantoso30.xsd minimal.akn30.xml
# → minimal.akn30.xml validates

# 3) 自检：确认没有联网（schema 只 import ./xml.xsd）
grep -n "schemaLocation" ../../schemas/akn31/akomantoso31-adapted.xsd
#   <xsd:import namespace="http://www.w3.org/XML/1998/namespace" schemaLocation="./xml.xsd"/>
```

实测结果（2026-11-07，libxml2 xmllint）：

| 用例 | 命令中的 schema | 结果 |
|---|---|---|
| `minimal.akn.xml`（ns 3.1） | `akomantoso30.xsd`（官方） | ❌ `No matching global declaration`（命名空间不匹配，预期） |
| `minimal.akn.xml`（ns 3.1） | `akomantoso31-adapted.xsd` | ✅ validates |
| `minimal.akn30.xml`（ns 3.0） | `akomantoso30.xsd`（官方） | ✅ validates |
| `minimal.akn30.xml`（ns 3.0） | `akomantoso31-adapted.xsd` | ❌ 命名空间不匹配（预期） |

---

## 12. 尚未拿到的东西 / 风险

1. **v2.0 规范正文未取得**：`.../v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.pdf`
   （47,752,703 B）在本次网络条件下多次连接重置/超时，未完整下载。
   但 **ZIP 中央目录已实测**（见 §1.1），从而确认了"官方包内没有 3.1 命名空间 XSD"这一关键事实；
   本文所有元素/属性事实来自**官方 XSD + 命名空间文档 + ZIP 条目清单**（可验证），
   规范正文中的推荐用法未覆盖。
2. **命名空间不一致**（§1.2）未解决；两种口径本仓库都支持。
3. **官方是否最终发布 `akomantoso31.xsd`**：目前 OS/CS01 包内都没有；
   若后续更新出现，应从 `.../akn-core/v2.0/os/part2-specs/schemas/` 重新核对并替换适配文件。
4. `akomantoso30.xml` 引用的 `modular.xsd` 官方包未随附；校验链路不使用它，无影响。
5. **3.1 差异**基于 XSD 比对，不含规范正文的"语义/示例"变化。

---

## 13. 来源 URL 清单（全部为官方/准官方）

标准与状态：
- OASIS LegalDocML TC 主页：https://www.oasis-open.org/committees/legaldocml/
- v2.0 OS 命名空间文档（2026-10-02，OASIS Standard）：
  https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/akn-core-v2.0-namespace.html
- v2.0 OS 规范 PDF：
  https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.pdf
- v2.0 OS schema 目录：
  https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/schemas/
- v2.0 OS 分发 ZIP：
  https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.zip
- v2.0 CS01 命名空间文档（2026-05-08）：
  https://docs.oasis-open.org/legaldocml/akn-core/v2.0/cs01/akn-core-v2.0-namespace.html
- v2.0 CS01 规范 PDF：
  https://docs.oasis-open.org/legaldocml/akn-core/v2.0/cs01/part2-specs/akn-core-v2.0-cs01-part2-specs.pdf
- OASIS 公告（v2.0 OS 2026-10-02 生效）：
  https://groups.oasis-open.org/discussion/akoma-ntoso-v2-0-part-2-akn-3-1-approved-as-an-oasis-standard-effective-2-oct
- Public review 公告（2026-07-16）：
  https://www.oasis-open.org/2026/07/16/invitation-to-comment-on-akoma-ntoso-v2-0-part-2-akn-3-1-before-call-for-consent-as-oasis-standard/

v1.0（对比基线）：
- v1.0 OS 命名空间文档（2018-08-29）：
  https://docs.oasis-open.org/legaldocml/akn-core/v1.0/os/akn-core-v1.0-namespace.html
- v1.0 Part 1 Vocabulary（OS）：
  https://docs.oasis-open.org/legaldocml/akn-core/v1.0/os/part1-vocabulary/akn-core-v1.0-os-part1-vocabulary.html
- v1.0 Part 2 schema 目录：
  https://docs.oasis-open.org/legaldocml/akn-core/v1.0/os/part2-specs/schemas/

辅助：
- Akoma Ntoso 社区站点（非 OASIS 官方）：http://akomantoso.info/
- 旧版 schema 仓库（2016–2017 快照，**不含 3.1**）：
  https://github.com/oasis-open/legaldocml-akomantoso

---

## 14. 给 Lead 的决策建议（摘要）

1. **命名空间**：Rust 生成器把 ns 做成常量；默认口径 A（ns 3.0 + `version="3.1"`，官方 XSD 可校验），
   CLI 提供 `--ns=3.1` 切到口径 B（本地适配 schema 校验）。输出报告里注明所用口径。
2. **根元素**：`doc` + `@name`；公文种类用 `FRBRsubtype` + `docType`（不要指望枚举）。
3. **未建模的公文要素**（发文机关标志/主送/抄送/版记/份号/密级/紧急程度/印章/版式）
   统一走 `proprietary`/`presentation` 的 `gbt:*` 扩展；建议 Lead 先定一份
   `urn:gbt9704:ext:1.0` 的小 XSD（可选，让 `lax` 生效），字段名与 GB/T 9704 条款号对应。
4. **版式不要塞进 AKN**：AKN 不负责分页/字体；分页信息最多用 `eop` 锚点。
5. **CI**：把 `xmllint --schema` 作为生成器测试门禁（两个口径各一组），并断言 `@eId` 唯一。

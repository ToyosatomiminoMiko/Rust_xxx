# gbt9704 — GB/T 9704-2012 党政机关公文格式 → Akoma Ntoso 生成器

把《党政机关公文格式》(GB/T 9704-2012) 的公文数据，按标准编排规则生成 **Akoma Ntoso 3.1**
(OASIS LegalDocML v2.0) XML。中间状态与存储格式就是 Akoma Ntoso：语义内容用 AKN 原生元素，
版式参数用 AKN 官方扩展点 (`proprietary` / `presentation`) 承载，后续 PDF 导出只从 AKN 读取。

## 阶段状态

| 阶段 | 内容 | 状态 |
| --- | --- | --- |
| 0 | 找到最新 Akoma Ntoso 标准；找到标准要求的字体；schema/字体落地并校验 | ✅ 完成 (见 `docs/`, `schemas/`, `fonts-manifest.json`) |
| 1 | Akoma Ntoso XML 生成 (输入 JSON → AKN 3.1，含全量标准项映射) | ✅ 完成 (本 crate) |
| 2 | 排版渲染 (字体/28 字网格折行/分页 + 版头/标题/正文/落款/印章/页码/版记 → PNG) | 🚧 通用格式可用 (附件、信函/命令/纪要特定版面、PDF 未做) |

## 技术路线

- **阶段 0/1 零第三方依赖**：JSON 解析/XML 序列化都是自实现；到阶段 2 渲染才引入两个纯 Rust
  crate：`ttf-parser`(OpenType 字形轮廓/度量) + `tiny-skia`(2D 光栅化/PNG 编码)，没有排版引擎或浏览器依赖；
- **标准即数据**：`src/gbt9704/` 把标准条款变成 Rust 常量/注册表，版式与映射都从同一份数据生成；
- **AKN 是唯一中间状态**：不再维护第二份私有格式；AKN 里既有语义，也有可重建版式的完整参数；
- **可校验**：生成结果用 OASIS 官方 XSD 离线校验（`xmllint`），并有 golden/映射一致性测试。

## 目录

```text
gbt9704/
├── src/
│   ├── gbt9704/          # 标准模型: geometry(版面几何), fonts(字体槽位), elements(要素注册表)
│   ├── model.rs          # 公文输入模型 + JSON 读写 + 校验
│   ├── akn/              # AKN 3.1 序列化 (xml.rs 是零依赖 XML writer)
│   ├── mapping.rs        # 44 个映射单元 + 77 项条款粒度覆盖 + 冲突说明
│   ├── json.rs           # 无依赖 JSON 解析/序列化
│   └── cli.rs            # 命令行
├── docs/
│   ├── gbt9704-2012-inventory.md        # 标准逐条清单(含 12 张式样图解读、23 条歧义)
│   ├── gbt9704-2012-elements.json       # 77 项机器可读要素清单
│   ├── gbt9704-2012-elements-registry.md# 本 crate 的要素注册表(生成)
│   ├── akoma-ntoso-v2.0.md              # AKN v2.0/AKN 3.1 事实来源、schema、能力矩阵
│   ├── mapping-gbt9704-akn.md           # 标准项 → AKN 全量映射 + 冲突清单(生成)
│   ├── fonts.md / fonts-manifest.json   # 字体要求、授权分析与开源替代/实测
│   └── figures/figure01.png … figure12.png  # 标准式样图(300 dpi)
├── schemas/akn31/        # 官方 AKN XSD + 本地 3.1 命名空间适配文件 (PROVENANCE.md 说明来源)
├── examples/akn/         # 最小合法 AKN 样例 (3.0/3.1 两种命名空间口径)
├── fonts/                # 按需下载的字体文件 (gitignore，不入库)
├── scripts/fetch-fonts.sh# 字体下载/校验脚本
└── tests/                # 单元 + schema 校验 + CLI + 映射一致性 + golden
```

## 快速开始

```sh
# 1. 输出示例输入
cargo run -p gbt9704 -- example -o example.json

# 2. 生成 AKN 3.1 XML（默认命名空间口径 A = akn/3.0 + version="3.1"，可被官方 XSD 校验）
cargo run -p gbt9704 -- build -i example.json -o example.akn.xml

# 3. 校验
xmllint --noout --schema gbt9704/schemas/akn31/akomantoso30.xsd example.akn.xml

# 4. 查看映射表/要素表/版面几何/字体要求
cargo run -p gbt9704 -- mapping
cargo run -p gbt9704 -- elements
cargo run -p gbt9704 -- geometry
cargo run -p gbt9704 -- fonts

# 5. 渲染为 PNG（阶段 2；字体先跑 scripts/fetch-fonts.sh）
#    多页时输出 <stem>-p1.png、<stem>-p2.png…，单页时直接用 -o 的文件名
cargo run -p gbt9704 -- render -i examples/example-honor-court-notice.json -o honor.png --dpi 200
#    不分页输出一张连续长页（适合截图/目视检查）
cargo run -p gbt9704 -- render -i examples/example-honor-court-notice.json -o honor-long.png --single
```

`render` 的字体查找顺序：`--fonts DIR` → 环境变量 `GBT9704_FONTS` → `./fonts` → crate 根 `fonts/`。
字体槽位与开源替代见 `docs/fonts.md`（标题/版头 小标宋≈Noto Serif CJK Bold、正文 仿宋≈FandolFang、
层次标题 黑体≈Noto Sans CJK、二级标题 楷体≈霞鹜文楷）。

`build --ns 3.1` 可切到 OASIS 命名空间文档口径（`akn/3.1`），用本地派生的
`akomantoso31-adapted.xsd` 校验，见下文“命名空间说明”。

## 输入 JSON

完整示例见 `examples/example-notice.json`（由 `gbt9704 example` 生成）；
`examples/example-honor-court-notice.json` 是一份带印章渲染效果的多页样例。字段：

| 字段 | 必填 | 说明 |
| --- | --- | --- |
| `doc_kind` | ✅ | 15 种文种：决议/决定/命令(令)/公报/公告/通告/意见/通知/通报/报告/请示/批复/议案/函/纪要 |
| `format` | | `standard`(默认) / `letter`(信函) / `order`(命令(令)) / `minutes`(纪要) |
| `issuer.names` | ✅ | 发文机关全称/规范化简称；联合行文按顺序排列，首个为主办机关 |
| `issuer.mark` | | 发文机关标志文本；缺省按格式推导（通用加“文件”，信函用机关名，命令加“命令”，纪要加“纪要”） |
| `title` | ✅ | 标题 |
| `doc_number` | | `{agency, year, serial}`，渲染为 `国办发〔2012〕1号`；可选 `text` 直接覆盖呈现文本（如“×××〔2026〕第001号”） |
| `order_number` | | 命令(令)的令号；缺省用 `doc_number.serial` 生成“第 N 号” |
| `copy_number` | | 份号（建议 6 位数字） |
| `secrecy` | | `{level: 绝密/机密/秘密, term: "1年"}`，渲染为“机密★1年” |
| `urgency` | | 紧急程度，如“特急” |
| `signers` | | `[{name, title}]` 签发人（上行文/命令） |
| `addressees` | | 主送机关名称数组；渲染时以“，”连接并以全角冒号结尾 |
| `addressees_in_record` | | 主送机关是否移至版记（首页放不下时） |
| `body` | ✅ | 段落数组；普通段 `{text}`，层次标题 `{level:1..4, num:"一、", text:"..."}` |
| `attachment_notes` | | 附件说明 `[{serial, name}]` |
| `signature` | ✅ | `{date:"YYYY-MM-DD", mode: with_seal/without_seal/signature_seal, issuer_names, seals:[{image, issuer, signer, title, number}]}`；可选 `date_text` 覆盖成文日期文本（如“二〇二六年十一月九日”） |
| `note` | | 附注，如“（此件公开发布）” |
| `attachments` | | 附件实体 `[{serial, title, paragraphs, standalone}]` |
| `cc` | | 抄送机关 |
| `printing` | | `{issuer, date}` 印发机关和印发日期 |
| `attendance` | | 纪要 `{present, absent, observers}` |
| `page.enabled` | | 是否编页码（默认 true） |
| `layout` | | 版心几何覆盖（默认严格等于标准；偏差会告警） |
| `frbr_base` | | 显式指定 FRBR 基础 IRI；缺省 `/cn/gbt9704/{文种}/{年}/{序号}` |

## AKN 设计

### 语义 → AKN 原生

- 标题 → `preface/p/docTitle` + `FRBRWork/FRBRname`
- 发文字号 → `preface/p/docNumber` + `FRBRWork/FRBRnumber`
- 成文日期 → `conclusions/p/date(@date=ISO)` + 三层 `FRBRdate`
- 文种 → `doc/@name` + `FRBRWork/FRBRsubtype`(ASCII slug) + `preface/p/docType`(中文，渲染隐藏)
- 发文机关 → `FRBRauthor` → `references/TLCOrganization`；署名 → `conclusions/p/signature`
- 签发人 → `workflow/step(@by,@as)` + `references/TLCPerson/TLCRole` + `preface/p/docProponent`
- 正文层次 → `mainBody/p` 与 `hcontainer[@name="level-1".."level-4"]/num/heading/intro/content`（4 层嵌套）
- 附件 → `attachments/attachment/doc`（内嵌独立 AKN 文档，独立 FRBR 链）
- 附注 → `meta/notes/note` + `conclusions/p[@gbt:role=annotation]`
- 密级/紧急程度 → `meta/classification/keyword`（语义近似，冲突见映射文档）
- 印发机关/日期 → `meta/publication`（语义近似）+ `proprietary` 版记

### AKN 无原生 → 扩展

- `akn:proprietary/gbt:gbt9704`：版记（抄送、移入版记的主送机关、印发信息）、令号
- `akn:presentation/gbt:presentation`：纸张/页边/版心/网格/字体槽位/颜色/页码/版记线/特定格式参数，
  以及 **44 个标准要素的完整呈现规格**（`gbt:element`，含条款号与规则文本）
- 可见的版头行（份号/密级/紧急程度/发文机关标志/发文字号/签发人/标题/主送机关）作为
  `preface/p` 内容，用 `gbt:role` 标记角色，渲染阶段据此定位
- 印章/签名章：`conclusions/p[@gbt:role=seal]` 占位 + `gbt:image` 等属性（图形资源）

## 标准覆盖与映射

- 映射表：`docs/mapping-gbt9704-akn.md`（由 `cargo run -p gbt9704 -- mapping` 生成）
- 44 个映射单元 + `docs/gbt9704-2012-elements.json` 的 77 项条款粒度覆盖，**无遗漏**；
- 分类：直接对应 6 / 语义近似 14 / 扩展承载 8 / 无对应(版式/印制) 16；
- 主要待讨论冲突（详见映射文档第 4 章）：
  1. 主送/抄送机关：AKN 3.1 **没有** `docAddressee`/`docRecipient`，只能 `preface/p` + `proprietary`；
  2. 版记（抄送/印发）在 AKN 中无容器位置（`attachments` 之后），只能 `proprietary`；
  3. 印章/签名章：图形与“下压/居中/不超版心”等版式约束无 AKN 语义；
  4. 份号/密级/紧急程度：无保密等级元素，份号只能扩展，密级/紧急程度只能借 `classification/keyword`；
  5. 页码、红头线、版记线、字体字号、版心、位置：纯版式，AKN 明确不负责，统一进 `presentation`；
  6. `publication` 语义是“公报发布”，不是“内部印发”，属近似映射；
  7. 命令(令)格式“无标题/无主送/无发文字号”与 AKN 通用 `doc` 的惯例存在张力（本模型允许标题必填）；
  8. 标题“梯形/菱形”、回行规则没有算法，只能由渲染层启发式处理；
  9. 信函/命令/纪要格式无专用根元素，用 `doc` + `FRBRsubtype` + `presentation` 参数表达。

## 字体

- 标准要求：2 号小标宋（标题）、3 号仿宋（正文/一般要素）、3 号黑体（密级/紧急/一级标题/附件标注）、
  3 号楷体（签发人/二级标题）、4 号仿宋（抄送/印发）、4 号半角宋体（页码）、红色小标宋（机关标志）。
- 官方字体（方正小标宋简体、仿宋_GB2312、楷体_GB2312 等）为商业/系统字体，**不随项目分发**；
- 已完成 10 款开源字体（121 MB）的下载与实测：Fandol 系 5 款 (GPL+字体例外)、Noto CJK 3 款 (OFL)、
  霞鹜文楷 (OFL)、朱雀仿宋 (OFL)；SHA256 10/10 通过、`fsType` 均可嵌入；
- 开源界**没有小标宋等价字体**（用 Noto Serif CJK SC Bold 近似）；仿宋首选 FandolFang（观感接近），
  OFL 可分发备选为朱雀仿宋（技术预览、缺全角数字）；GB18030 生僻字没有一款全覆盖，需逐字回退并报警；
- 获取/校验：`bash scripts/fetch-fonts.sh` / `--check`；授权与实测细节见 `docs/fonts.md`；
- 机器可读槽位见 `src/gbt9704/fonts.rs` 与 `fonts-manifest.json`（`body_fangsong` 等 9 个槽位名）；
  阶段 2 直接按槽位嵌入字体，需注意 Noto/Fandol 是 CFF、霞鹜/朱雀是 TTF/glyf，PDF 库支持要先冒烟测试。

## 标准与 schema 版本

- 公文格式：**GB/T 9704-2012**（2012-06-29 发布，2012-07-01 实施；代替 GB/T 9704-1999）
- Akoma Ntoso：**v2.0 Part 2 (AKN 3.1)**，OASIS Standard（2026-10-02 生效）
- 命名空间说明：OASIS v2.0 命名空间文档声明 `.../ns/akn/3.1`，但官方分发包里实际发布的
  XSD 仍是 `akomantoso30.xsd`（`targetNamespace=.../ns/akn/3.0`，只把根 `@version` 固定为 `3.1`）。
  这是官方包内部不一致，详见 `docs/akoma-ntoso-v2.0.md`。本生成器：
  - **默认口径 A**：`akn/3.0` + `version="3.1"`，用**官方** XSD 校验；
  - `--ns 3.1`：`akn/3.1` + `version="3.1"`，用本地派生的 `akomantoso31-adapted.xsd` 校验（明确标注非官方）；
  - 两种口径元素/属性集合完全相同，切换只改一行 XML 声明。

## 测试

```sh
cargo test -p gbt9704          # 单元 + 集成 + schema 校验 + golden + CLI
cargo clippy -p gbt9704        # 可选
```

- schema 校验依赖 `xmllint`（libxml2）；没有时相关测试打印提示并跳过；
- golden 基准：`tests/golden/example.akn.xml`；重新生成命令见文件头注释。

## 阶段 2（渲染，通用格式已可用）

已实现（`src/render/`，输入直接用阶段 1 的 `GbDocument`，暂未绕回 AKN 读回）：

- 版面：A4 210×297、天头 37/订口 28、版心 156×225、22 行 × 28 字；字宽按 156/28 mm 紧缩
  （≈0.987 倍名义字号），行距 225/22 mm；
- 版头：份号/密级★期限/紧急程度（3 号黑体）、发文机关标志（红色小标宋，名称与“文件”分行、
  字号自适应版心宽，式样图里“文件”比名称大）、发文字号（居中/上行文左空一字+签发人）、
  红色分隔线（发文字号下 4 mm、0.6 mm 粗）；
- 主体：2 号小标宋标题（禁则折行 + 两行时按“词意完整/梯形”挑断点）、主送机关、3 号仿宋正文
  （首行缩进 2 字、回行顶格、禁则/标点悬挂、两端对齐）、附注、发文机关署名、
  成文日期（右空四字）、红色圆形印章（外圆 + 环形机关名称 + 五角星 + 编号，半透明压盖署名与日期）；
- 分页：落款/印章若在末页放不下会把末尾几行挪到新页，保证印章不越出版心、末页有正文；
- 版记（抄送/印发机关日期，首末粗线 + 中间细线）与页码（4 号宋体“— N —”，单页右/双页左，版心下 7 mm）；
- 输出 PNG（tiny-skia 自带的 png 编码），`--single` 输出不分页的连续长页。

未做/待补：附件（另面编排、附件说明）、横排表格、信函/命令/纪要三种特定格式版面、
直接 PDF 输出与 CMYK 油墨、从 AKN 读回版式参数（目前渲染层直接吃阶段 1 的模型，`presentation`
里的参数已足够重建，但尚未接线）。

## 来源

- GB/T 9704-2012 原文 PDF（本仓库 `docs/gbt9704-2012-inventory.md` 的提取基准）
- OASIS LegalDocML：
  - <https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.pdf>
  - <https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/akn-core-v2.0-namespace.html>
  - <https://docs.oasis-open.org/legaldocml/akn-core/v1.0/os/part1-vocabulary/akn-core-v1.0-os-part1-vocabulary.html>
- 其他参考文献与逐条链接见 `docs/gbt9704-2012-inventory.md`、`docs/akoma-ntoso-v2.0.md`。

# AKN 3.1 schema vendoring 来源与版权说明（PROVENANCE）

本目录（`gbt9704/schemas/akn31/`）中的文件用于离线校验 GB/T 9704 生成器的输出。
**所有文件均为人工下载后原样保存，未做内容改写**；唯一的派生物是
`akomantoso31-adapted.xsd`，其生成方式见下文第 4 节。

- 下载日期：2026-11-07（Asia/Shanghai）
- 下载环境：本机 curl / Python，直连 `docs.oasis-open.org`（无镜像）
- 校验工具：`xmllint --schema`（libxml2），离线只依赖本目录文件

---

## 1. 官方来源（OASIS Standard, Akoma Ntoso Version 2.0, Part 2: Specifications of AKN 3.1）

| 内容 | URL | 状态 |
|---|---|---|
| OS 命名空间文档（2026-10-02） | https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/akn-core-v2.0-namespace.html | OASIS Standard |
| OS 规范 PDF | https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.pdf | OASIS Standard（47,752,703 bytes，本仓库未随附） |
| OS schema 目录 | https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/schemas/ | 目录索引可访问 |
| OS 分发 ZIP | https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/akn-core-v2.0-os-part2-specs.zip | 32,847,536 bytes（本仓库未随附） |
| CS01 命名空间文档（2026-05-08） | https://docs.oasis-open.org/legaldocml/akn-core/v2.0/cs01/akn-core-v2.0-namespace.html | Committee Specification 01 |
| CS01 规范 PDF | https://docs.oasis-open.org/legaldocml/akn-core/v2.0/cs01/part2-specs/akn-core-v2.0-cs01-part2-specs.pdf | CS01 |

实际抓取到本目录的三个文件，全部来自 **OS schema 目录**
`https://docs.oasis-open.org/legaldocml/akn-core/v2.0/os/part2-specs/schemas/`
（其文件名与 CS01 目录完全一致，且 **CS01 与 OS 的这两个 XSD 字节级相同**，已用 `cmp` 验证）。

> 注意：`docs.oasis-open.org` 在本次下载中多次出现连接重置/超时，47 MB 的 PDF 未能完整落地；
> schema 三个小文件（251 KB / 8 KB / 153 KB）均完整下载并通过 `xmllint` 加载。
> **32.8 MB 的 OS 分发 ZIP 只下载了末尾 256 KB**，用 Python 解析 ZIP 中央目录后确认其条目为：
> `akn-core-v2.0-os-part2-specs.pdf`（压缩 32,786,713 B / 解压 47,752,703 B）、
> `schemas/akomantoso30.xsd`（27,598 / 257,230 B）、`schemas/akomantoso30.xml`（30,304 / 156,824 B）、
> `schemas/xml.xsd`（2,405 / 8,836 B）。即：**整个 OASIS v2.0 OS 分发包里不存在
> `akomantoso31.xsd`，也不存在其它 3.1 命名空间的 schema。**

## 2. 随附文件与 SHA256（2026-11-07 计算）

| 文件 | 字节数 | SHA256 | 说明 |
|---|---:|---|---|
| `akomantoso30.xsd` | 257,230 | `8f1fe3e187ee5e6f5bc9dfb013f7d9195de6516f077ce10726bedf6260f8b70a` | 官方 OS XSD 原件（targetNamespace = `.../ns/akn/3.0`） |
| `akomantoso30.xml` | 156,824 | `2878d439f401a051398e31efda88d9f070e6c5c951feb80d5d330ac2bf67ad4e` | 官方 "schema-driving XML"（模块组合定义，非实例文档） |
| `xml.xsd` | 8,836 | `61960fb3131e38022caad5360e2f33a3382578ab3c80cd58bd74320ede61b20c` | 官方随附的 W3C XML 命名空间 schema（`xml:lang/xml:space/xml:id`） |
| `akomantoso31-adapted.xsd` | 257,230 | `bf73a2ac5891cdca67af1ac320427a2c19f39c19394c7ab2f310555b826dbc04` | **本仓库本地派生**，见第 4 节；不是 OASIS 交付物 |

`akomantoso30.xsd` 头部自述：

```
Akoma Ntoso main schema
Final release 11/06/2025 - Akoma Ntoso 3.0
targetNamespace="http://docs.oasis-open.org/legaldocml/ns/akn/3.0"
根元素 akomaNtoso 的 @version 属性: fixed="3.1"
```

即：**官方 v2.0 包里随附的 XSD 文件名仍叫 `akomantoso30.xsd`、命名空间仍是 `.../ns/akn/3.0`，
但根元素 `@version` 已被固定为 `"3.1"`**。这一点与 OASIS 命名空间文档声明的
`http://docs.oasis-open.org/legaldocml/ns/akn/3.1` 不一致，详见
`gbt9704/docs/akoma-ntoso-v2.0.md` 的“版本与状态”和“风险”章节。

## 3. 依赖关系（离线校验所需文件）

`akomantoso30.xsd` / `akomantoso31-adapted.xsd` 的唯一外部依赖：

```xml
<xsd:import namespace="http://www.w3.org/XML/1998/namespace" schemaLocation="./xml.xsd"/>
```

`xml.xsd` 位于同目录，因此 `xmllint --schema akomantoso31-adapted.xsd ...` 全程离线、不访问网络。
`akomantoso30.xml` 不被 XSD 引用，仅为官方分发内容的完整保留（`xsi:noNamespaceSchemaLocation="./modular.xsd"`，
`modular.xsd` 官方包内未随附；本目录**不**引用它，故不影响校验）。

## 4. `akomantoso31-adapted.xsd` 的派生方式与性质

由于 OASIS v2.0 OS 包的 XSD 仍指向 `.../ns/akn/3.0`，而 OASIS 命名空间文档声明 AKN 3.1 的命名空间为
`.../ns/akn/3.1`，为了能对“按官方命名空间文档写的 3.1 实例”做离线校验，本仓库做了一次**纯机械重定向**：

```sh
sed -e 's#http://docs.oasis-open.org/legaldocml/ns/akn/3\.0#http://docs.oasis-open.org/legaldocml/ns/akn/3.1#g' \
    akomantoso30.xsd > akomantoso31-adapted.xsd
```

全文仅改写两处（默认 `xmlns` 与 `targetNamespace`），其余 100% 与官方 OS 原件逐字节相同
（`diff <(sed ...) akomantoso31-adapted.xsd` 为空）。

**它只是本地校验适配层，不是 OASIS 规范文件**，不得对外标注为“OASIS AKN 3.1 schema”。
如果需要与任何第三方系统互操作，请优先使用官方 `akomantoso30.xsd`（ns 3.0）并在文档里
显式声明命名空间口径；两种口径的取舍见 `docs/akoma-ntoso-v2.0.md`。

## 5. 版权与再分发（OASIS 声明原文要点）

`akomantoso30.xsd` 的 `<xsd:annotation>` 中写明：

> This schema is provided by OASIS, the Copyright Holder, under licence cc-by 4.0, that can also be
> found at the URL https://creativecommons.org/licenses/by/4.0/legalcode

因此：

- 本目录对 schema 的再分发属于 OASIS 明示许可（CC-BY 4.0）范围内的使用；
- 再分发时须保留版权/许可声明与出处（即本文件与 XSD 内嵌 annotation 均不得删除）；
- `akomantoso30.xml`、`xml.xsd` 为同一 OASIS 分发包内的伴随文件；`xml.xsd` 源自 W3C
  XML Namespace Recommendation（W3C Software and Document License）。
- 建议在对外产物中标注：*Contains schema files © OASIS Open, licensed under CC-BY 4.0,
  downloaded from docs.oasis-open.org on 2026-11-07.*

## 6. 已知限制

1. 47 MB 的 OS/CS01 规范 PDF 未完整下载、32 MB ZIP 仅取末尾 256 KB（但已足以解析完整中央目录）。
   文档中的语义描述来自 XSD 内嵌 `xsd:documentation`（官方 schema 自带）、官方命名空间文档与
   ZIP 条目清单，并以规范 URL 作为权威引用。
2. `akomantoso30.xsd` 的 `@version` 固定值 `"3.1"` 与 targetNamespace `3.0` 的组合是官方包现状，
   本仓库不做修正，只记录。
3. 未随附 `modular.xsd`（官方 v2.0 包内亦未提供；`akomantoso30.xml` 引用它，但校验链路不依赖它）。

# gbt9704 文档索引

本目录是阶段 0/1 的事实来源与交付说明。所有文档由调研队友与 Lead 在 2026-11-07 产出/校验。

## 标准侧

| 文件 | 内容 | 生成/维护 |
|---|---|---|
| [`gbt9704-2012-inventory.md`](gbt9704-2012-inventory.md) | GB/T 9704-2012 条款 1~11 逐条清单：12 张式样图解读、23 条歧义、图测交叉核对、AKN 冲突要点、Rust 常量建议 | 人工调研 (Lead 复核) |
| [`gbt9704-2012-elements.json`](gbt9704-2012-elements.json) | 77 项机器可读格式要素（id/name/part/source_clause/required/font/position/spacing/rules/figures） | 人工调研 |
| [`gbt9704-2012-elements-registry.md`](gbt9704-2012-elements-registry.md) | 本 crate 的 44 个映射单元注册表（由代码生成） | `cargo run -p gbt9704 -- elements` |
| [`figures/`](figures/) | 12 张标准式样图（`pdftoppm -r 300` 抽取） | 调研产物 |

## Akoma Ntoso 侧

| 文件 | 内容 | 生成/维护 |
|---|---|---|
| [`akoma-ntoso-v2.0.md`](akoma-ntoso-v2.0.md) | AKN v2.0/AKN 3.1 版本与状态、命名空间矛盾、根元素/内容模型、meta/FRBR、层级、附件、扩展机制、3.0→3.1 差异、能力矩阵、校验方法 | 人工调研 (Lead 复核) |
| [`mapping-gbt9704-akn.md`](mapping-gbt9704-akn.md) | 44 个映射单元 + 77 项条款粒度覆盖 + 冲突/待讨论清单 | `cargo run -p gbt9704 -- mapping` |
| [`../schemas/akn31/PROVENANCE.md`](../schemas/akn31/PROVENANCE.md) | 官方 XSD 来源/SHA256/版权、本地 3.1 适配文件说明 | 调研产物 |
| [`../examples/akn/`](../examples/akn/) | 最小合法 AKN 样例（官方 3.0 命名空间 / 本地 3.1 命名空间） | 调研产物 |

## 字体侧

| 文件 | 内容 | 生成/维护 |
|---|---|---|
| [`fonts.md`](fonts.md) | 标准字体要求、授权分析、开源替代、实测 family/覆盖、阶段 2 嵌入注意 | 调研产物 |
| [`../fonts-manifest.json`](../fonts-manifest.json) | 机器可读字体槽位/文件/URL/license/SHA256 | 调研产物 |
| [`../scripts/fetch-fonts.sh`](../scripts/fetch-fonts.sh) | 多源下载 + SHA256 校验脚本（字体文件不入库） | 调研产物 |

## 权威顺序

1. 标准原文 PDF 与 OASIS 官方文件；
2. `gbt9704-2012-inventory.md` / `gbt9704-2012-elements.json` / `akoma-ntoso-v2.0.md`（调研记录，标注了来源）；
3. 代码侧注册表与映射表（`src/gbt9704/elements.rs`、`src/mapping.rs`）；
4. 生成物（`*-registry.md`、`mapping-gbt9704-akn.md`）。

修改标准相关数值时：先改代码侧注册表，再重新生成文档，最后跑 `cargo test -p gbt9704`。

//! gbt9704: GB/T 9704-2012《党政机关公文格式》→ Akoma Ntoso 3.1 生成器.
//!
//! 阶段 0: 标准/字体调研与 schema vendoring(见 docs/);
//! 阶段 1: 从 JSON 输入生成 AKN 3.1 XML(本 crate);
//! 阶段 2: PDF 导出(后续).
//!
//! 模块:
//! - `gbt9704`: 标准的机器可读模型(版面几何/字体/要素注册表);
//! - `model`: 公文输入模型与 JSON 读写;
//! - `akn`: AKN 3.1 序列化;
//! - `mapping`: 标准要素 → AKN 映射表与文档生成;
//! - `json`: 零依赖 JSON 解析/序列化;
//! - `cli`: 命令行入口.

pub mod akn;
pub mod cli;
pub mod gbt9704;
pub mod json;
pub mod mapping;
pub mod model;

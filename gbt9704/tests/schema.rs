//! 集成测试: 生成的 AKN XML 必须能通过官方/适配 schema 的离线校验.
//!
//! 两个命名空间口径(A/B)各对应一个 schema:
//! - 口径 A(default): `schemas/akn31/akomantoso30.xsd` (官方 v2.0 OS 原件, ns akn/3.0)
//! - 口径 B: `schemas/akn31/akomantoso31-adapted.xsd` (本地派生, ns akn/3.1)
//!
//! 依赖 `xmllint`(libxml2); 缺失时打印提示并跳过, 不把环境依赖伪装成通过.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use gbt9704::akn::{NsMode, serialize_with};
use gbt9704::model::example_document;

static COUNTER: AtomicU64 = AtomicU64::new(0);

fn schema_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("schemas/akn31")
}

fn temp_xml(xml: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path =
        std::env::temp_dir().join(format!("gbt9704_schema_{}_{index}.xml", std::process::id()));
    fs::write(&path, xml).expect("写入临时 XML 失败");
    path
}

fn xmllint_available() -> bool {
    Command::new("xmllint")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn validate(xml: &str, schema_file: &str) -> Result<(), String> {
    let schema = schema_dir().join(schema_file);
    assert!(
        schema.exists(),
        "缺少 vendored schema: {} (见 schemas/akn31/PROVENANCE.md)",
        schema.display()
    );
    let xml_path = temp_xml(xml);
    let output = Command::new("xmllint")
        .arg("--noout")
        .arg("--schema")
        .arg(&schema)
        .arg(&xml_path)
        .output()
        .map_err(|error| format!("无法运行 xmllint: {error}"))?;
    let _ = fs::remove_file(&xml_path);
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[test]
fn generated_xml_is_well_formed_and_validates_against_both_namespace_schemas() {
    if !xmllint_available() {
        eprintln!("跳过 schema 校验: 未安装 xmllint");
        return;
    }
    let document = example_document();
    for (mode, schema) in [
        (NsMode::Official30, "akomantoso30.xsd"),
        (NsMode::Namespace31, "akomantoso31-adapted.xsd"),
    ] {
        let xml = serialize_with(&document, mode).expect("示例应可序列化");
        if let Err(stderr) = validate(&xml, schema) {
            panic!("口径 {} 使用 {} 校验失败:\n{}", mode.id(), schema, stderr);
        }
        // 额外确认 XML 良构(即使 schema 校验被跳过).
        let xml_path = temp_xml(&xml);
        let status = Command::new("xmllint")
            .arg("--noout")
            .arg(&xml_path)
            .status()
            .expect("运行 xmllint 失败");
        let _ = fs::remove_file(&xml_path);
        assert!(status.success(), "生成的 XML 不是良构 XML");
    }
}

#[test]
fn vendored_schema_is_offline_only() {
    // schema 只允许 import 同目录的 xml.xsd; 出现 http(s) 的 schemaLocation 会导致离线 CI 失败.
    for file in ["akomantoso30.xsd", "akomantoso31-adapted.xsd"] {
        let text = fs::read_to_string(schema_dir().join(file)).expect("读取 schema 失败");
        for line in text.lines() {
            if let Some(index) = line.find("schemaLocation=") {
                let tail = &line[index + "schemaLocation=".len()..];
                assert!(
                    tail.contains("./xml.xsd"),
                    "{file} 含非本地 schemaLocation: {line}"
                );
            }
        }
    }
}

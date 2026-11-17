//! CLI 冒烟测试: 子命令可运行, 退出码符合约定, build 产物可解析.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use gbt9704::json;

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_gbt9704")
}

fn run(args: &[&str]) -> Output {
    Command::new(binary())
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("无法运行 gbt9704: {error}"))
}

fn temp_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("gbt9704_cli_{}_{name}", std::process::id()))
}

#[test]
fn help_and_version() {
    let help = run(&["--help"]);
    assert!(help.status.success());
    let text = String::from_utf8_lossy(&help.stdout);
    assert!(text.contains("GB/T 9704-2012"));
    assert!(text.contains("build"));

    let version = run(&["--version"]);
    assert!(version.status.success());
    assert!(String::from_utf8_lossy(&version.stdout).contains("gbt9704"));
}

#[test]
fn example_emits_valid_json() {
    let output = run(&["example"]);
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    let value = json::parse(&text).expect("example 输出必须是合法 JSON");
    assert_eq!(
        value.get("doc_kind").and_then(|value| value.as_str()),
        Some("通知")
    );
}

#[test]
fn build_writes_akn_xml_and_handles_namespace_modes() {
    let json_path = temp_path("input.json");
    fs::write(&json_path, gbt9704::model::EXAMPLE_JSON).unwrap();

    for (mode, namespace) in [
        ("3.0", "http://docs.oasis-open.org/legaldocml/ns/akn/3.0"),
        ("3.1", "http://docs.oasis-open.org/legaldocml/ns/akn/3.1"),
    ] {
        let xml_path = temp_path(&format!("output_{mode}.akn.xml"));
        let output = run(&[
            "build",
            "-i",
            json_path.to_str().unwrap(),
            "-o",
            xml_path.to_str().unwrap(),
            "--ns",
            mode,
        ]);
        assert!(
            output.status.success(),
            "build --ns {mode} 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let xml = fs::read_to_string(&xml_path).unwrap();
        assert!(xml.contains(namespace));
        assert!(xml.contains("version=\"3.1\""));
        let _ = fs::remove_file(&xml_path);
    }
    let _ = fs::remove_file(&json_path);
}

#[test]
fn build_rejects_invalid_input_with_nonzero_exit() {
    let path = temp_path("bad.json");
    fs::write(&path, r#"{"doc_kind":"不存在"}"#).unwrap();
    let output = run(&["build", "-i", path.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2));
    let _ = fs::remove_file(&path);
}

#[test]
fn read_only_commands_succeed() {
    for args in [
        vec!["geometry"],
        vec!["fonts"],
        vec!["elements"],
        vec!["mapping"],
        vec!["mapping", "--format", "json"],
    ] {
        let output = run(&args);
        assert!(
            output.status.success(),
            "{args:?} 失败: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!output.stdout.is_empty(), "{args:?} 没有输出");
    }
}

#[test]
fn unknown_command_returns_usage_error() {
    let output = run(&["no-such-command"]);
    assert_eq!(output.status.code(), Some(2));
}

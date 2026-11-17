//! 映射一致性测试: 标准侧清单(77 项)与映射表必须一一覆盖.

use std::fs;
use std::path::Path;

use gbt9704::gbt9704::elements;
use gbt9704::json;
use gbt9704::mapping::{self, GRANULAR_COVERAGE, MAPPINGS};

#[test]
fn mapping_table_has_no_gaps() {
    mapping::check_consistency().expect("映射表自检失败");
    for element in elements::ELEMENTS {
        assert!(
            mapping::find(element.id).is_some(),
            "要素 {} 没有映射",
            element.id
        );
    }
}

#[test]
fn granular_coverage_matches_standard_inventory_json() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/gbt9704-2012-elements.json");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("读取 {} 失败: {error}", path.display()));
    let value = json::parse(&text).expect("标准要素清单必须是合法 JSON");
    let items = value
        .get("elements")
        .and_then(|value| value.as_array())
        .expect("JSON 缺少 elements 数组");
    assert_eq!(items.len(), 77, "标准侧细分要素数量应为 77");

    let mut std_ids = Vec::new();
    for item in items {
        let id = item
            .get("id")
            .and_then(|value| value.as_str())
            .expect("要素缺少 id");
        std_ids.push(id.to_string());
    }
    let covered: std::collections::HashMap<&str, &str> =
        GRANULAR_COVERAGE.iter().copied().collect();
    assert_eq!(covered.len(), 77, "粒度覆盖表应有 77 项");
    for id in &std_ids {
        assert!(
            covered.contains_key(id.as_str()),
            "标准细分要素 {id} 没有粒度覆盖记录"
        );
        let parent = covered[id.as_str()];
        assert!(
            mapping::find(parent).is_some(),
            "细分要素 {id} 指向不存在的映射单元 {parent}"
        );
    }
    // 反向: 覆盖表里不应有清单之外的 id.
    for (id, _) in GRANULAR_COVERAGE {
        assert!(std_ids.iter().any(|item| item == id), "覆盖表多出 {id}");
    }
}

#[test]
fn rendered_documents_mention_every_standard_item() {
    let markdown = mapping::render_markdown();
    assert!(markdown.contains("全量映射表"));
    assert!(markdown.contains("冲突与待讨论清单"));
    for element in elements::ELEMENTS {
        assert!(markdown.contains(element.id), "映射文档缺少 {}", element.id);
    }
    for (std_id, _) in GRANULAR_COVERAGE {
        assert!(markdown.contains(std_id), "映射文档缺少粒度项 {std_id}");
    }
    let json_text = mapping::render_json();
    let value = json::parse(&json_text).expect("映射 JSON 必须合法");
    assert_eq!(
        value
            .get("summary")
            .and_then(|summary| summary.get("total"))
            .and_then(|total| total.as_f64()),
        Some(MAPPINGS.len() as f64)
    );
    assert_eq!(
        value
            .get("summary")
            .and_then(|summary| summary.get("granular"))
            .and_then(|count| count.as_f64()),
        Some(77.0)
    );
}

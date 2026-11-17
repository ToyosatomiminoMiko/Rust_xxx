//! Golden 测试: 示例输入的 AKN 输出必须与仓库内基准文件一致.
//!
//! 基准文件更新方式:
//!   cargo run -p gbt9704 -- example -o .scratch/example.json
//!   cargo run -p gbt9704 -- build -i .scratch/example.json -o tests/golden/example.akn.xml
//! 变更序列化逻辑时, 应同时 review 基准 diff, 而不是无脑覆盖.

use gbt9704::akn::serialize;
use gbt9704::model::example_document;

#[test]
fn example_output_matches_golden() {
    let document = example_document();
    let xml = serialize(&document).expect("示例应可序列化");
    let golden = include_str!("golden/example.akn.xml");
    assert_eq!(
        xml, golden,
        "AKN 输出与 tests/golden/example.akn.xml 不一致; 若为有意变更, 请重新生成基准并 review"
    );
}

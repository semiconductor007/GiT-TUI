//! 风险页面测试：验证风险等级和原因列表的展示文本。

use gitinsight_rs::analytics::{RiskLevel, RiskReport};
use gitinsight_rs::ui::risk::risk_rows;

#[test]
fn risk_rows_include_level_and_reasons() {
    let report = RiskReport {
        risk_level: RiskLevel::High,
        reasons: vec![
            "Bus Factor = 1，关键贡献者数量偏少".to_owned(),
            "单一贡献者拥有 85.0% 的提交，知识集中度较高".to_owned(),
        ],
    };

    let rows = risk_rows(&report);

    assert_eq!(rows[0], "风险等级: 高");
    assert!(rows.iter().any(|row| row == "风险原因:"));
    assert!(rows.iter().any(|row| row.contains("Bus Factor = 1")));
    assert!(rows.iter().any(|row| row.contains("85.0%")));
    assert!(rows.iter().any(|row| row.contains("知识集中度较高")));
}

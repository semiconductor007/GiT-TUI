//! 风险页面测试：验证风险等级和原因列表的展示文本。

use gitinsight_rs::analytics::{RiskLevel, RiskReport};
use gitinsight_rs::ui::risk::risk_rows;

#[test]
fn risk_rows_include_level_and_reasons() {
    let report = RiskReport {
        risk_level: RiskLevel::High,
        reasons: vec![
            "Bus Factor = 1".to_owned(),
            "Single contributor owns 85.0% of commits".to_owned(),
        ],
    };

    let rows = risk_rows(&report);

    assert_eq!(rows[0], "风险等级: 高");
    assert!(rows.iter().any(|row| row == "风险原因:"));
    assert!(rows.iter().any(|row| row.contains("Bus Factor = 1")));
    assert!(rows.iter().any(|row| row.contains("85.0%")));
}

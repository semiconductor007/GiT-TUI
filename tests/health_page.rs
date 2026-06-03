//! 健康度页面测试：验证评分、风险等级和说明文本展示。

use gitinsight_rs::analytics::{BusFactorReport, HealthScore, RiskLevel};
use gitinsight_rs::ui::health::health_rows;

#[test]
fn health_rows_include_score_bus_factor_and_risk() {
    let score = HealthScore::new(100, 70, 35, 25);
    let bus_factor = BusFactorReport {
        bus_factor: 1,
        top_contributors: vec!["Tom".to_owned()],
        risk_level: RiskLevel::High,
    };

    let rows = health_rows(&score, &bus_factor);

    assert!(rows.iter().any(|row| row == "健康度评分: 57/100"));
    assert!(
        rows.iter()
            .any(|row| row == "仓库阶段: Established Repository")
    );
    assert!(rows.iter().any(|row| row == "Bus Factor: 1"));
    assert!(rows.iter().any(|row| row == "风险等级: 高"));
}

#[test]
fn health_rows_include_score_explanation() {
    let score = HealthScore::new(100, 70, 35, 0);
    let bus_factor = BusFactorReport::default();

    let rows = health_rows(&score, &bus_factor);

    assert!(rows.iter().any(|row| row.starts_with("评分说明:")));
    assert!(rows.iter().any(|row| row.contains("提交活跃度较好")));
    assert!(rows.iter().any(|row| row.contains("文件热点缺失")));
}

#[test]
fn health_rows_mark_early_stage_repository() {
    let score = HealthScore::with_stage(
        gitinsight_rs::analytics::RepositoryStage::EarlyStage,
        35,
        35,
        35,
        35,
    );
    let bus_factor = BusFactorReport::default();

    let rows = health_rows(&score, &bus_factor);

    assert!(
        rows.iter()
            .any(|row| row == "仓库阶段: Early Stage Repository")
    );
    assert!(rows.iter().any(|row| row.contains("提交样本较少")));
    assert!(rows.iter().any(|row| row.contains("保守评分")));
}

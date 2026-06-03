//! 风险分析测试：验证风险等级和可解释风险原因生成。

use chrono::DateTime;
use gitinsight_rs::analytics::{
    BusFactorReport, FileHotspot, HealthScore, RiskAnalyzer, RiskLevel,
};
use gitinsight_rs::models::ContributorStats;

#[test]
fn high_risk_report_explains_multiple_reasons() {
    let mut tom = ContributorStats::new("Tom", "tom@example.com");
    tom.commit_count = 17;
    tom.ownership_percent = 85.0;
    let mut alice = ContributorStats::new("Alice", "alice@example.com");
    alice.commit_count = 3;
    alice.ownership_percent = 15.0;

    let bus_factor = BusFactorReport {
        bus_factor: 1,
        top_contributors: vec!["Tom".to_owned()],
        risk_level: RiskLevel::High,
    };
    let health_score = HealthScore::new(40, 35, 35, 50);
    let hotspots = hotspots_with_counts(&[70, 20, 10]);

    let report = RiskAnalyzer::from_analysis(&[tom, alice], &bus_factor, &health_score, &hotspots);

    assert_eq!(report.risk_level, RiskLevel::High);
    assert!(
        report
            .reasons
            .iter()
            .any(|reason| reason == "Bus Factor = 1，关键贡献者数量偏少")
    );
    assert!(report.reasons.iter().any(|reason| reason.contains("85.0%")));
    assert!(
        report
            .reasons
            .iter()
            .any(|reason| reason.contains("Top 3 热点文件"))
    );
    assert!(
        report
            .reasons
            .iter()
            .any(|reason| reason.contains("健康度低于 60 分"))
    );
}

#[test]
fn low_risk_report_has_positive_fallback_reason() {
    let mut alice = ContributorStats::new("Alice", "alice@example.com");
    alice.commit_count = 4;
    alice.ownership_percent = 40.0;
    let mut bob = ContributorStats::new("Bob", "bob@example.com");
    bob.commit_count = 3;
    bob.ownership_percent = 30.0;
    let mut tom = ContributorStats::new("Tom", "tom@example.com");
    tom.commit_count = 3;
    tom.ownership_percent = 30.0;

    let bus_factor = BusFactorReport {
        bus_factor: 3,
        top_contributors: vec!["Alice".to_owned(), "Bob".to_owned(), "Tom".to_owned()],
        risk_level: RiskLevel::Low,
    };
    let health_score = HealthScore::new(100, 100, 100, 100);
    let hotspots = hotspots_with_counts(&[10, 10, 10, 10, 10, 10, 10]);

    let report =
        RiskAnalyzer::from_analysis(&[alice, bob, tom], &bus_factor, &health_score, &hotspots);

    assert_eq!(report.risk_level, RiskLevel::Low);
    assert_eq!(report.reasons, vec!["未发现明显仓库风险信号"]);
}

fn hotspots_with_counts(change_counts: &[usize]) -> Vec<FileHotspot> {
    let timestamp = DateTime::from_timestamp(1_735_689_600, 0).expect("timestamp should be valid");

    change_counts
        .iter()
        .enumerate()
        .map(|(index, change_count)| {
            let mut hotspot = FileHotspot::new(format!("src/file_{index}.rs"));
            hotspot.change_count = *change_count;
            hotspot.last_modified = timestamp;
            hotspot.score = 1.0;
            hotspot
        })
        .collect()
}

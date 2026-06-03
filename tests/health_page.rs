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

    assert!(rows.iter().any(|row| row == "Health Score: 57/100"));
    assert!(rows.iter().any(|row| row == "Bus Factor: 1"));
    assert!(rows.iter().any(|row| row == "Risk Level: High"));
}

#[test]
fn health_rows_include_score_explanation() {
    let score = HealthScore::new(100, 70, 35, 0);
    let bus_factor = BusFactorReport::default();

    let rows = health_rows(&score, &bus_factor);

    assert!(rows.iter().any(|row| row.starts_with("Explanation:")));
    assert!(rows.iter().any(|row| row.contains("activity strong")));
    assert!(rows.iter().any(|row| row.contains("hotspots missing")));
}

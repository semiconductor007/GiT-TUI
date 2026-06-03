pub mod bus_factor;
pub mod contributors;
pub mod health;
pub mod hotspot;
pub mod manager;
pub mod overview;
pub mod timeline;

pub use bus_factor::{BusFactorAnalyzer, BusFactorReport, RiskLevel};
pub use contributors::ContributorAnalyzer;
pub use health::{HealthAnalyzer, HealthScore};
pub use hotspot::{FileHotspot, HotspotAnalyzer};
pub use manager::{AnalysisManager, AnalysisSnapshot};
pub use overview::OverviewAnalyzer;
pub use timeline::{DEFAULT_TIMELINE_LIMIT, TimelineAnalyzer};

use crate::git::GitRepository;
use crate::utils::{AppError, Result};

fn validate_repository(repo: &GitRepository) -> Result<()> {
    if repo.repository_name()?.trim().is_empty() {
        return Err(AppError::AnalysisError(
            "repository name cannot be empty".to_owned(),
        ));
    }

    Ok(())
}

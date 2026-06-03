use crate::git::{Analyzer, GitRepository};
use crate::models::RepositorySummary;
use crate::utils::Result;

#[derive(Debug, Clone, Default)]
pub struct OverviewAnalyzer;

impl Analyzer for OverviewAnalyzer {
    type Output = RepositorySummary;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output> {
        repo.summary()
    }
}

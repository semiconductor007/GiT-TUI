use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepositorySummary {
    pub name: String,
    pub path: PathBuf,
    pub total_commits: usize,
    pub total_branches: usize,
    pub total_tags: usize,
    pub total_contributors: usize,
    pub total_files: usize,
    pub total_loc: usize,
}

impl RepositorySummary {
    pub fn new(name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            ..Self::default()
        }
    }
}

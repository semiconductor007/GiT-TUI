use crate::git::GitRepository;
use crate::utils::Result;

pub trait Analyzer {
    type Output;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output>;
}

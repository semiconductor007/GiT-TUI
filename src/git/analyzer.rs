//! 分析器 Trait：定义所有仓库分析模块的统一接口。

use crate::git::GitRepository;
use crate::utils::Result;

pub trait Analyzer {
    type Output;

    fn analyze(&self, repo: &GitRepository) -> Result<Self::Output>;
}

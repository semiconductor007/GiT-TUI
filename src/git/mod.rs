//! Git 模块入口：导出仓库访问层、提交工具和分析器 Trait。

pub mod analyzer;
pub mod commit;
pub mod repository;

pub use analyzer::Analyzer;
pub use repository::GitRepository;

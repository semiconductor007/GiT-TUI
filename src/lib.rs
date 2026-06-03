//! 库入口：统一导出应用、Git 数据层、分析器、模型、UI 和工具模块。

pub mod analytics;
pub mod app;
pub mod git;
pub mod models;
pub mod ui;
pub mod utils;

pub use utils::error::{AppError, Result};

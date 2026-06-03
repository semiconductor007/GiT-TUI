//! 工具模块入口：导出统一错误类型和时间处理函数。

pub mod error;
pub mod time;

pub use error::{AppError, Result};

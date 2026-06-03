use thiserror::Error;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("git error: {0}")]
    GitError(#[from] git2::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("analysis error: {0}")]
    AnalysisError(String),
}

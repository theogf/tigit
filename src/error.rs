use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitDiffError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("Failed to find repository")]
    RepositoryNotFound,

    #[error("Failed to detect main branch (tried: main, master, origin/main, origin/master)")]
    MainBranchNotFound,

    #[error("No differences found between HEAD and {0}")]
    NoDifferences(String),

    #[error("Working directory is dirty. Please commit or stash changes first.")]
    DirtyWorkingDirectory,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("UTF-8 conversion error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("Invalid diff format: {0}")]
    InvalidDiff(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, GitDiffError>;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GitDiffError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error(
        "Not a git repository (or any of the parent directories)\n\n\
        Hints:\n\
        - Run this command from within a git repository\n\
        - Or specify a repository path with --path <PATH>\n\
        - Initialize a git repository with 'git init' if needed"
    )]
    RepositoryNotFound,

    #[error("No differences found between HEAD and {0}")]
    NoDifferences(String),

    #[allow(dead_code)]
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

// Library interface for git-main-diff
// This allows integration tests to import modules

pub mod diff;
pub mod error;
pub mod git;

// Re-export commonly used types
pub use diff::types::*;
pub use error::*;

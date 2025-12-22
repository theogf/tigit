use crate::error::{GitDiffError, Result};
use git2::{BranchType, Repository};
use std::path::Path;

/// Open a git repository at the given path or discover it
pub fn open_repository(path: Option<&Path>) -> Result<Repository> {
    let repo = if let Some(p) = path {
        Repository::open(p)?
    } else {
        Repository::discover(".")?
    };
    Ok(repo)
}

/// Detect the main branch name by trying common variants
pub fn detect_main_branch(repo: &Repository) -> Result<String> {
    let branch_candidates = vec![
        ("main", BranchType::Local),
        ("master", BranchType::Local),
        ("origin/main", BranchType::Remote),
        ("origin/master", BranchType::Remote),
    ];

    for (name, branch_type) in branch_candidates {
        if repo.find_branch(name, branch_type).is_ok() {
            return Ok(name.to_string());
        }
    }

    Err(GitDiffError::MainBranchNotFound)
}

/// Get HEAD and main branch commits
pub fn get_commits<'a>(
    repo: &'a Repository,
    main_branch: &str,
) -> Result<(git2::Commit<'a>, git2::Commit<'a>)> {
    // Get HEAD commit
    let head = repo.head()?;
    let head_commit = head.peel_to_commit()?;

    // Determine branch type based on name
    let branch_type = if main_branch.starts_with("origin/") {
        BranchType::Remote
    } else {
        BranchType::Local
    };

    // Get main branch commit
    let main_branch_ref = repo.find_branch(main_branch, branch_type)?;
    let main_commit = main_branch_ref.get().peel_to_commit()?;

    Ok((head_commit, main_commit))
}

/// Check if working directory is clean
#[allow(dead_code)]
pub fn is_working_directory_clean(repo: &Repository) -> Result<bool> {
    let statuses = repo.statuses(None)?;
    Ok(statuses.is_empty())
}

use crate::error::{GitDiffError, Result};
use git2::{BranchType, ErrorCode, Repository};
use std::path::Path;

/// Open a git repository at the given path or discover it
pub fn open_repository(path: Option<&Path>) -> Result<Repository> {
    let repo = if let Some(p) = path {
        Repository::open(p).map_err(|e| match e.code() {
            ErrorCode::NotFound => GitDiffError::RepositoryNotFound,
            _ => {
                let path_str = p.display();
                GitDiffError::Other(format!(
                    "Failed to open git repository at '{}': {}\n\n\
                    Hints:\n\
                    - Ensure the path exists and is accessible\n\
                    - Check that it's a valid git repository (contains a .git directory)\n\
                    - Verify you have permission to access the directory",
                    path_str, e
                ))
            }
        })?
    } else {
        Repository::discover(".").map_err(|e| match e.code() {
            ErrorCode::NotFound => GitDiffError::RepositoryNotFound,
            _ => GitDiffError::Other(format!(
                "Failed to find git repository: {}\n\n\
                    Hints:\n\
                    - Run this command from within a git repository\n\
                    - Or specify a repository path with --path <PATH>\n\
                    - Initialize a git repository with 'git init' if needed",
                e
            )),
        })?
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

    // List available branches for better error message
    let mut local_branches = Vec::new();
    if let Ok(branches) = repo.branches(Some(BranchType::Local)) {
        for branch in branches.flatten() {
            let (branch, _) = branch;
            if let Some(name) = branch.name().ok().flatten() {
                local_branches.push(name.to_string());
            }
        }
    }

    let available = if local_branches.is_empty() {
        "No branches found".to_string()
    } else {
        format!("Available branches: {}", local_branches.join(", "))
    };

    Err(GitDiffError::Other(format!(
        "Failed to detect main branch (tried: main, master, origin/main, origin/master)\n\n\
        {}\n\n\
        Hints:\n\
        - Specify the branch explicitly with --branch <BRANCH>\n\
        - Ensure you have commits on your main branch\n\
        - If using a different default branch name, specify it with --branch",
        available
    )))
}

/// Get HEAD and main branch commits
pub fn get_commits<'a>(
    repo: &'a Repository,
    main_branch: &str,
) -> Result<(git2::Commit<'a>, git2::Commit<'a>)> {
    // Get HEAD commit
    let head = repo.head().map_err(|e| match e.code() {
        ErrorCode::UnbornBranch => GitDiffError::Other(
            "No commits yet in this repository\n\n\
            Hints:\n\
            - Make at least one commit before using this tool\n\
            - Use 'git commit' to create your first commit"
                .to_string(),
        ),
        ErrorCode::NotFound => GitDiffError::Other(
            "HEAD reference not found\n\n\
            Hints:\n\
            - This might indicate a corrupted repository\n\
            - Try running 'git status' to check repository health"
                .to_string(),
        ),
        _ => GitDiffError::Other(format!("Failed to read HEAD: {}", e)),
    })?;

    let head_commit = head.peel_to_commit().map_err(|e| {
        GitDiffError::Other(format!(
            "Failed to resolve HEAD to a commit: {}\n\n\
            Hints:\n\
            - Ensure you have at least one commit in the repository\n\
            - Check that HEAD is not detached in an invalid state",
            e
        ))
    })?;

    // Determine branch type based on name
    let branch_type = if main_branch.starts_with("origin/") {
        BranchType::Remote
    } else {
        BranchType::Local
    };

    // Get main branch commit
    let main_branch_ref = repo.find_branch(main_branch, branch_type).map_err(|e| {
        GitDiffError::Other(format!(
            "Branch '{}' not found\n\n\
            Hints:\n\
            - Check the branch name is correct (case-sensitive)\n\
            - Use 'git branch -a' to list all available branches\n\
            - For remote branches, ensure you've run 'git fetch'\n\
            - Specify a different branch with --branch <BRANCH>\n\n\
            Error: {}",
            main_branch, e
        ))
    })?;

    let main_commit = main_branch_ref.get().peel_to_commit().map_err(|e| {
        GitDiffError::Other(format!(
            "Failed to resolve branch '{}' to a commit: {}",
            main_branch, e
        ))
    })?;

    Ok((head_commit, main_commit))
}

/// Check if working directory is clean
#[allow(dead_code)]
pub fn is_working_directory_clean(repo: &Repository) -> Result<bool> {
    let statuses = repo.statuses(None)?;
    Ok(statuses.is_empty())
}

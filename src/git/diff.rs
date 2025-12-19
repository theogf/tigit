use crate::diff::types::{DiffLine, DiffSet, FileDiff, FileStatus, Hunk, LineOrigin};
use crate::error::{GitDiffError, Result};
use git2::{Commit, Delta, Diff, DiffOptions, Repository};
use std::cell::RefCell;

/// Get diff between main branch and HEAD
pub fn get_diff<'repo>(
    repo: &'repo Repository,
    main_commit: &Commit,
    head_commit: &Commit,
) -> Result<Diff<'repo>> {
    let main_tree = main_commit.tree()?;
    let head_tree = head_commit.tree()?;

    let mut opts = DiffOptions::new();
    opts.context_lines(3);

    let diff = repo.diff_tree_to_tree(Some(&main_tree), Some(&head_tree), Some(&mut opts))?;

    Ok(diff)
}

/// Parse git2::Diff into our DiffSet structure
pub fn parse_diff(
    diff: &Diff,
    head_commit_id: String,
    main_commit_id: String,
    main_branch_name: String,
) -> Result<DiffSet> {
    let mut diff_set = DiffSet::new(head_commit_id, main_commit_id, main_branch_name);
    let current_file: RefCell<Option<FileDiff>> = RefCell::new(None);
    let hunk_counter: RefCell<usize> = RefCell::new(0);

    diff.foreach(
        &mut |delta, _progress| {
            // Start a new file
            if let Some(file) = current_file.borrow_mut().take() {
                diff_set.files.push(file);
            }

            let path = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .and_then(|p| p.to_str())
                .unwrap_or("<unknown>")
                .to_string();

            let status = match delta.status() {
                Delta::Added => FileStatus::Added,
                Delta::Deleted => FileStatus::Deleted,
                Delta::Modified => FileStatus::Modified,
                _ => FileStatus::Modified,
            };

            *current_file.borrow_mut() = Some(FileDiff::new(path, status));
            *hunk_counter.borrow_mut() = 0;

            true
        },
        None,
        Some(&mut |_delta, hunk| {
            if let Some(ref mut file) = *current_file.borrow_mut() {
                let header = String::from_utf8_lossy(hunk.header()).to_string();
                let counter = *hunk_counter.borrow();

                let hunk_obj = Hunk::new(
                    counter,
                    header.trim().to_string(),
                    hunk.old_start(),
                    hunk.old_lines(),
                    hunk.new_start(),
                    hunk.new_lines(),
                );

                file.hunks.push(hunk_obj);
                *hunk_counter.borrow_mut() += 1;
            }

            true
        }),
        Some(&mut |_delta, _hunk, line| {
            if let Some(ref mut file) = *current_file.borrow_mut() {
                if let Some(current_hunk) = file.hunks.last_mut() {
                    let origin = LineOrigin::from_git2(line.origin());
                    let content = String::from_utf8_lossy(line.content()).to_string();

                    let old_lineno = line.old_lineno();
                    let new_lineno = line.new_lineno();

                    let diff_line = DiffLine::new(origin, content, old_lineno, new_lineno);
                    current_hunk.lines.push(diff_line);
                }
            }

            true
        }),
    )?;

    // Don't forget the last file
    if let Some(file) = current_file.borrow_mut().take() {
        diff_set.files.push(file);
    }

    Ok(diff_set)
}

/// Extract and parse diff in one call
pub fn extract_diff_set(
    repo: &Repository,
    main_branch: &str,
) -> Result<DiffSet> {
    use super::repository::get_commits;

    let (head_commit, main_commit) = get_commits(repo, main_branch)?;

    let head_id = head_commit.id().to_string();
    let main_id = main_commit.id().to_string();

    let diff = get_diff(repo, &main_commit, &head_commit)?;
    let diff_set = parse_diff(&diff, head_id, main_id, main_branch.to_string())?;

    if diff_set.is_empty() {
        return Err(GitDiffError::NoDifferences(main_branch.to_string()));
    }

    Ok(diff_set)
}

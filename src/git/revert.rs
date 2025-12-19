use crate::diff::types::{DiffSet, LineOrigin};
use crate::error::Result;
use git2::Repository;
use std::path::Path;

/// Generate a reverse patch from selected hunks
/// This swaps additions and deletions to reverse the changes
pub fn create_reverse_patch(diff_set: &DiffSet) -> Result<String> {
    let mut patch = String::new();

    for file in &diff_set.files {
        let selected_hunks: Vec<_> = file.hunks.iter().filter(|h| h.selected).collect();

        if selected_hunks.is_empty() {
            continue;
        }

        // Diff header
        patch.push_str(&format!(
            "diff --git a/{} b/{}\n",
            file.path, file.path
        ));
        patch.push_str(&format!("--- a/{}\n", file.path));
        patch.push_str(&format!("+++ b/{}\n", file.path));

        for hunk in selected_hunks {
            // Reverse the hunk header: swap old and new ranges
            patch.push_str(&format!(
                "@@ -{},{} +{},{} @@\n",
                hunk.new_start, hunk.new_lines, hunk.old_start, hunk.old_lines
            ));

            // Reverse the lines: swap + and -
            for line in &hunk.lines {
                let reversed_origin = match line.origin {
                    LineOrigin::Addition => LineOrigin::Deletion,
                    LineOrigin::Deletion => LineOrigin::Addition,
                    LineOrigin::Context => LineOrigin::Context,
                };

                let prefix = reversed_origin.to_char();
                // Ensure line ends with newline if the content doesn't have one
                let content = if line.content.ends_with('\n') {
                    &line.content
                } else {
                    &format!("{}\n", line.content)
                };
                patch.push_str(&format!("{}{}", prefix, content));
            }
        }
    }

    Ok(patch)
}

/// Apply reversions by staging the changes
pub fn apply_and_stage_reversions(repo: &Repository, diff_set: &DiffSet) -> Result<usize> {
    let patch = create_reverse_patch(diff_set)?;

    if patch.is_empty() {
        return Ok(0);
    }

    // Write patch to a temporary file
    let temp_dir = std::env::temp_dir();
    let patch_path = temp_dir.join(format!("git-main-diff-{}.patch", std::process::id()));
    std::fs::write(&patch_path, &patch)?;

    // Apply the patch using git apply
    let output = std::process::Command::new("git")
        .arg("apply")
        .arg("--index") // Apply to both working directory and index (stage)
        .arg(&patch_path)
        .current_dir(repo.workdir().unwrap())
        .output()?;

    // Clean up temp file
    let _ = std::fs::remove_file(&patch_path);

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(crate::error::GitDiffError::Other(format!(
            "Failed to apply patch: {}",
            error
        )));
    }

    Ok(diff_set.selected_hunks())
}

/// Alternative: Stage specific file paths (less precise than patch-based approach)
pub fn stage_files(repo: &Repository, file_paths: &[&str]) -> Result<()> {
    let mut index = repo.index()?;

    for path in file_paths {
        index.add_path(Path::new(path))?;
    }

    index.write()?;
    Ok(())
}

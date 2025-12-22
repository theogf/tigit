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
        patch.push_str(&format!("diff --git a/{} b/{}\n", file.path, file.path));
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
    let patch_path = temp_dir.join(format!("tigit-{}.patch", std::process::id()));
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

        // Provide helpful error messages for common cases
        let message = if error.contains("patch does not apply") {
            "Reversions already applied or working directory has changed. The diff shown is now stale.".to_string()
        } else if error.contains("already exists in working directory") {
            "Changes already reverted. Try refreshing or restart the application.".to_string()
        } else {
            format!("Failed to apply patch: {}", error.trim())
        };

        return Err(crate::error::GitDiffError::Other(message));
    }

    Ok(diff_set.selected_hunks())
}

/// Alternative: Stage specific file paths (less precise than patch-based approach)
#[allow(dead_code)]
pub fn stage_files(repo: &Repository, file_paths: &[&str]) -> Result<()> {
    let mut index = repo.index()?;

    for path in file_paths {
        index.add_path(Path::new(path))?;
    }

    index.write()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::{DiffLine, DiffSet, FileDiff, FileStatus, Hunk, LineOrigin};

    #[test]
    fn test_create_reverse_patch_empty() {
        let diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let patch = create_reverse_patch(&diff_set).unwrap();
        assert!(patch.is_empty());
    }

    #[test]
    fn test_create_reverse_patch_no_selection() {
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk = Hunk::new(0, "@@ -10,3 +10,3 @@".to_string(), 10, 3, 10, 3);
        hunk.lines.push(DiffLine::new(
            LineOrigin::Context,
            "line 10\n".to_string(),
            Some(10),
            Some(10),
        ));
        file.hunks.push(hunk);
        diff_set.files.push(file);

        let patch = create_reverse_patch(&diff_set).unwrap();
        assert!(patch.is_empty());
    }

    #[test]
    fn test_create_reverse_patch_simple_addition() {
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk = Hunk::new(0, "@@ -10,2 +10,3 @@".to_string(), 10, 2, 10, 3);
        hunk.selected = true;

        hunk.lines.push(DiffLine::new(
            LineOrigin::Context,
            "line 10\n".to_string(),
            Some(10),
            Some(10),
        ));
        hunk.lines.push(DiffLine::new(
            LineOrigin::Addition,
            "new line\n".to_string(),
            None,
            Some(11),
        ));
        hunk.lines.push(DiffLine::new(
            LineOrigin::Context,
            "line 11\n".to_string(),
            Some(11),
            Some(12),
        ));

        file.hunks.push(hunk);
        diff_set.files.push(file);

        let patch = create_reverse_patch(&diff_set).unwrap();

        assert!(patch.contains("diff --git a/test.txt b/test.txt"));
        assert!(patch.contains("--- a/test.txt"));
        assert!(patch.contains("+++ b/test.txt"));
        assert!(patch.contains("@@ -10,3 +10,2 @@"));
        assert!(patch.contains(" line 10"));
        assert!(patch.contains("-new line"));
        assert!(patch.contains(" line 11"));
    }

    #[test]
    fn test_create_reverse_patch_simple_deletion() {
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk = Hunk::new(0, "@@ -10,3 +10,2 @@".to_string(), 10, 3, 10, 2);
        hunk.selected = true;

        hunk.lines.push(DiffLine::new(
            LineOrigin::Context,
            "line 10\n".to_string(),
            Some(10),
            Some(10),
        ));
        hunk.lines.push(DiffLine::new(
            LineOrigin::Deletion,
            "deleted line\n".to_string(),
            Some(11),
            None,
        ));
        hunk.lines.push(DiffLine::new(
            LineOrigin::Context,
            "line 12\n".to_string(),
            Some(12),
            Some(11),
        ));

        file.hunks.push(hunk);
        diff_set.files.push(file);

        let patch = create_reverse_patch(&diff_set).unwrap();

        assert!(patch.contains("@@ -10,2 +10,3 @@"));
        assert!(patch.contains(" line 10"));
        assert!(patch.contains("+deleted line"));
        assert!(patch.contains(" line 12"));
    }

    #[test]
    fn test_create_reverse_patch_multiple_hunks() {
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);

        // First hunk - selected
        let mut hunk1 = Hunk::new(0, "@@ -1,1 +1,2 @@".to_string(), 1, 1, 1, 2);
        hunk1.selected = true;
        hunk1.lines.push(DiffLine::new(
            LineOrigin::Context,
            "line 1\n".to_string(),
            Some(1),
            Some(1),
        ));
        hunk1.lines.push(DiffLine::new(
            LineOrigin::Addition,
            "new line\n".to_string(),
            None,
            Some(2),
        ));

        // Second hunk - not selected
        let mut hunk2 = Hunk::new(1, "@@ -10,1 +11,1 @@".to_string(), 10, 1, 11, 1);
        hunk2.selected = false;
        hunk2.lines.push(DiffLine::new(
            LineOrigin::Addition,
            "another line\n".to_string(),
            None,
            Some(11),
        ));

        // Third hunk - selected
        let mut hunk3 = Hunk::new(2, "@@ -20,1 +22,1 @@".to_string(), 20, 1, 22, 1);
        hunk3.selected = true;
        hunk3.lines.push(DiffLine::new(
            LineOrigin::Deletion,
            "removed\n".to_string(),
            Some(20),
            None,
        ));

        file.hunks.push(hunk1);
        file.hunks.push(hunk2);
        file.hunks.push(hunk3);
        diff_set.files.push(file);

        let patch = create_reverse_patch(&diff_set).unwrap();

        // Should contain first and third hunks, but not second
        assert!(patch.contains("@@ -1,2 +1,1 @@"));
        assert!(!patch.contains("@@ -11,1 +10,1 @@")); // Second hunk should not be in patch
        assert!(patch.contains("@@ -22,1 +20,1 @@"));
    }

    #[test]
    fn test_create_reverse_patch_multiple_files() {
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        // First file
        let mut file1 = FileDiff::new("file1.txt".to_string(), FileStatus::Modified);
        let mut hunk1 = Hunk::new(0, "@@ -1,1 +1,2 @@".to_string(), 1, 1, 1, 2);
        hunk1.selected = true;
        hunk1.lines.push(DiffLine::new(
            LineOrigin::Addition,
            "added\n".to_string(),
            None,
            Some(2),
        ));
        file1.hunks.push(hunk1);

        // Second file
        let mut file2 = FileDiff::new("file2.txt".to_string(), FileStatus::Modified);
        let mut hunk2 = Hunk::new(0, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1);
        hunk2.selected = true;
        hunk2.lines.push(DiffLine::new(
            LineOrigin::Deletion,
            "removed\n".to_string(),
            Some(5),
            None,
        ));
        file2.hunks.push(hunk2);

        diff_set.files.push(file1);
        diff_set.files.push(file2);

        let patch = create_reverse_patch(&diff_set).unwrap();

        assert!(patch.contains("diff --git a/file1.txt b/file1.txt"));
        assert!(patch.contains("diff --git a/file2.txt b/file2.txt"));
    }

    #[test]
    fn test_reverse_patch_line_without_newline() {
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk = Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1);
        hunk.selected = true;

        // Line without trailing newline
        hunk.lines.push(DiffLine::new(
            LineOrigin::Addition,
            "no newline".to_string(),
            None,
            Some(1),
        ));

        file.hunks.push(hunk);
        diff_set.files.push(file);

        let patch = create_reverse_patch(&diff_set).unwrap();

        // Should add newline
        assert!(patch.contains("-no newline\n"));
    }
}

use super::types::{DiffSet, FileDiff};

/// Toggle selection for a specific hunk in a file
pub fn toggle_hunk_selection(diff_set: &mut DiffSet, file_idx: usize, hunk_idx: usize) {
    if let Some(file) = diff_set.files.get_mut(file_idx) {
        if let Some(hunk) = file.hunks.get_mut(hunk_idx) {
            hunk.toggle_selection();
        }
    }
}

/// Select all hunks in a specific file
pub fn select_all_in_file(file: &mut FileDiff) {
    for hunk in &mut file.hunks {
        hunk.selected = true;
    }
}

/// Deselect all hunks in a specific file
pub fn deselect_all_in_file(file: &mut FileDiff) {
    for hunk in &mut file.hunks {
        hunk.selected = false;
    }
}

/// Get indices of all selected hunks across all files
#[allow(dead_code)]
pub fn get_selected_hunks(diff_set: &DiffSet) -> Vec<(usize, usize)> {
    let mut selected = Vec::new();
    for (file_idx, file) in diff_set.files.iter().enumerate() {
        for (hunk_idx, hunk) in file.hunks.iter().enumerate() {
            if hunk.selected {
                selected.push((file_idx, hunk_idx));
            }
        }
    }
    selected
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diff::types::{DiffSet, FileStatus, Hunk};

    #[test]
    fn test_toggle_hunk_selection() {
        let mut diff_set = DiffSet::new(
            "head".to_string(),
            "main".to_string(),
            "main".to_string(),
        );

        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        file.hunks.push(Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1));
        file.hunks.push(Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1));
        diff_set.files.push(file);

        // Initially not selected
        assert!(!diff_set.files[0].hunks[0].selected);

        // Toggle on
        toggle_hunk_selection(&mut diff_set, 0, 0);
        assert!(diff_set.files[0].hunks[0].selected);

        // Toggle off
        toggle_hunk_selection(&mut diff_set, 0, 0);
        assert!(!diff_set.files[0].hunks[0].selected);
    }

    #[test]
    fn test_toggle_invalid_indices() {
        let mut diff_set = DiffSet::new(
            "head".to_string(),
            "main".to_string(),
            "main".to_string(),
        );

        // Should not panic with invalid indices
        toggle_hunk_selection(&mut diff_set, 0, 0);
        toggle_hunk_selection(&mut diff_set, 99, 99);
    }

    #[test]
    fn test_select_all_in_file() {
        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        file.hunks.push(Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1));
        file.hunks.push(Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1));
        file.hunks.push(Hunk::new(2, "@@ -10,1 +10,1 @@".to_string(), 10, 1, 10, 1));

        assert!(!file.hunks[0].selected);
        assert!(!file.hunks[1].selected);
        assert!(!file.hunks[2].selected);

        select_all_in_file(&mut file);

        assert!(file.hunks[0].selected);
        assert!(file.hunks[1].selected);
        assert!(file.hunks[2].selected);
    }

    #[test]
    fn test_deselect_all_in_file() {
        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk1 = Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1);
        let mut hunk2 = Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1);
        hunk1.selected = true;
        hunk2.selected = true;

        file.hunks.push(hunk1);
        file.hunks.push(hunk2);

        deselect_all_in_file(&mut file);

        assert!(!file.hunks[0].selected);
        assert!(!file.hunks[1].selected);
    }

    #[test]
    fn test_get_selected_hunks() {
        let mut diff_set = DiffSet::new(
            "head".to_string(),
            "main".to_string(),
            "main".to_string(),
        );

        let mut file1 = FileDiff::new("file1.txt".to_string(), FileStatus::Modified);
        let mut hunk1 = Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1);
        let hunk2 = Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1);
        hunk1.selected = true;
        file1.hunks.push(hunk1);
        file1.hunks.push(hunk2);

        let mut file2 = FileDiff::new("file2.txt".to_string(), FileStatus::Modified);
        let mut hunk3 = Hunk::new(0, "@@ -10,1 +10,1 @@".to_string(), 10, 1, 10, 1);
        hunk3.selected = true;
        file2.hunks.push(hunk3);

        diff_set.files.push(file1);
        diff_set.files.push(file2);

        let selected = get_selected_hunks(&diff_set);

        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0], (0, 0)); // file1, hunk1
        assert_eq!(selected[1], (1, 0)); // file2, hunk3
    }

    #[test]
    fn test_get_selected_hunks_empty() {
        let diff_set = DiffSet::new(
            "head".to_string(),
            "main".to_string(),
            "main".to_string(),
        );

        let selected = get_selected_hunks(&diff_set);
        assert!(selected.is_empty());
    }
}

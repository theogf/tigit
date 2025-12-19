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

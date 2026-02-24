use super::types::{DiffSet, FileDiff, Hunk, LineOrigin};

/// Toggle selection for a specific hunk in a file
pub fn toggle_hunk_selection(diff_set: &mut DiffSet, file_idx: usize, hunk_idx: usize) {
    if let Some(file) = diff_set.files.get_mut(file_idx)
        && let Some(hunk) = file.hunks.get_mut(hunk_idx)
    {
        hunk.toggle_selection();
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

/// Select all hunks across all files
pub fn select_all_global(diff_set: &mut DiffSet) {
    for file in &mut diff_set.files {
        for hunk in &mut file.hunks {
            hunk.selected = true;
        }
    }
}

/// Deselect all hunks across all files
pub fn deselect_all_global(diff_set: &mut DiffSet) {
    for file in &mut diff_set.files {
        for hunk in &mut file.hunks {
            hunk.selected = false;
        }
    }
}

/// Split a hunk into two at the first context-line gap between change groups.
/// Returns true if a split was performed, false if the hunk cannot be split.
pub fn split_hunk(diff_set: &mut DiffSet, file_idx: usize, hunk_idx: usize) -> bool {
    let Some(file) = diff_set.files.get(file_idx) else {
        return false;
    };
    let Some(hunk) = file.hunks.get(hunk_idx) else {
        return false;
    };

    let lines = hunk.lines.clone();
    let was_selected = hunk.selected;

    // Find the index of the first context line that comes after a changed line
    // AND has more changed lines after it (i.e. it's a gap between two change groups).
    let mut seen_change = false;
    let mut split_point: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        match line.origin {
            LineOrigin::Addition | LineOrigin::Deletion => {
                seen_change = true;
            }
            LineOrigin::Context => {
                if seen_change {
                    let more_changes = lines[i + 1..]
                        .iter()
                        .any(|l| matches!(l.origin, LineOrigin::Addition | LineOrigin::Deletion));
                    if more_changes {
                        split_point = Some(i);
                        break;
                    }
                }
            }
        }
    }

    // Fallback: if no context gap exists, split at the first deletion that
    // comes after a run of additions (adjacent change-block boundary).
    let split_idx = match split_point {
        Some(i) => i,
        None => {
            let mut seen_additions = false;
            let mut boundary = None;
            for (i, line) in lines.iter().enumerate() {
                match line.origin {
                    LineOrigin::Addition => seen_additions = true,
                    LineOrigin::Deletion if seen_additions => {
                        boundary = Some(i);
                        break;
                    }
                    _ => {}
                }
            }
            match boundary {
                Some(i) => i,
                None => return false, // single change block, cannot split
            }
        }
    };

    // For context-gap splits, split at the midpoint of the context run so
    // each sub-hunk retains some context. For boundary splits the split_idx
    // is already right at the start of the second change block.
    let context_run_len = lines[split_idx..]
        .iter()
        .take_while(|l| matches!(l.origin, LineOrigin::Context))
        .count();

    let mid = split_idx + context_run_len / 2;

    let lines1 = lines[..mid].to_vec();
    let lines2 = lines[mid..].to_vec();

    // Derive hunk parameters from the line numbers already stored in DiffLine
    let calc = |ls: &[super::types::DiffLine]| -> Option<(u32, u32, u32, u32)> {
        let old_start = ls.iter().find_map(|l| l.old_lineno)?;
        let new_start = ls.iter().find_map(|l| l.new_lineno)?;
        let old_lines = ls.iter().filter(|l| l.old_lineno.is_some()).count() as u32;
        let new_lines = ls.iter().filter(|l| l.new_lineno.is_some()).count() as u32;
        Some((old_start, old_lines, new_start, new_lines))
    };

    let (os1, ol1, ns1, nl1) = match calc(&lines1) {
        Some(p) => p,
        None => return false,
    };
    let (os2, ol2, ns2, nl2) = match calc(&lines2) {
        Some(p) => p,
        None => return false,
    };

    let header1 = format!("@@ -{},{} +{},{} @@", os1, ol1, ns1, nl1);
    let header2 = format!("@@ -{},{} +{},{} @@", os2, ol2, ns2, nl2);

    let mut h1 = Hunk::new(hunk_idx, header1, os1, ol1, ns1, nl1);
    h1.lines = lines1;
    h1.selected = was_selected;

    let mut h2 = Hunk::new(hunk_idx + 1, header2, os2, ol2, ns2, nl2);
    h2.lines = lines2;
    h2.selected = was_selected;

    let file = &mut diff_set.files[file_idx];
    file.hunks.remove(hunk_idx);
    file.hunks.insert(hunk_idx, h2);
    file.hunks.insert(hunk_idx, h1);

    true
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
    use crate::diff::types::{DiffLine, DiffSet, FileStatus, Hunk, LineOrigin};

    #[test]
    fn test_toggle_hunk_selection() {
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        file.hunks
            .push(Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1));
        file.hunks
            .push(Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1));
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
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        // Should not panic with invalid indices
        toggle_hunk_selection(&mut diff_set, 0, 0);
        toggle_hunk_selection(&mut diff_set, 99, 99);
    }

    #[test]
    fn test_select_all_in_file() {
        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        file.hunks
            .push(Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1));
        file.hunks
            .push(Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1));
        file.hunks
            .push(Hunk::new(2, "@@ -10,1 +10,1 @@".to_string(), 10, 1, 10, 1));

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
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

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
        let diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());

        let selected = get_selected_hunks(&diff_set);
        assert!(selected.is_empty());
    }

    #[test]
    fn test_split_hunk_context_gap() {
        // Two change groups separated by context lines
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());
        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk = Hunk::new(0, "@@ -1,5 +1,5 @@".to_string(), 1, 5, 1, 5);
        hunk.lines = vec![
            DiffLine::new(LineOrigin::Deletion, "old a\n".to_string(), Some(1), None),
            DiffLine::new(LineOrigin::Addition, "new a\n".to_string(), None, Some(1)),
            DiffLine::new(LineOrigin::Context, "ctx\n".to_string(), Some(2), Some(2)),
            DiffLine::new(LineOrigin::Deletion, "old b\n".to_string(), Some(3), None),
            DiffLine::new(LineOrigin::Addition, "new b\n".to_string(), None, Some(3)),
        ];
        file.hunks.push(hunk);
        diff_set.files.push(file);

        assert!(split_hunk(&mut diff_set, 0, 0));
        assert_eq!(diff_set.files[0].hunks.len(), 2);
    }

    #[test]
    fn test_split_hunk_adjacent_blocks() {
        // Two change blocks with no context between them — boundary split
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());
        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk = Hunk::new(0, "@@ -1,4 +1,4 @@".to_string(), 1, 4, 1, 4);
        hunk.lines = vec![
            DiffLine::new(LineOrigin::Deletion, "old a\n".to_string(), Some(1), None),
            DiffLine::new(LineOrigin::Addition, "new a\n".to_string(), None, Some(1)),
            DiffLine::new(LineOrigin::Deletion, "old b\n".to_string(), Some(2), None),
            DiffLine::new(LineOrigin::Addition, "new b\n".to_string(), None, Some(2)),
        ];
        file.hunks.push(hunk);
        diff_set.files.push(file);

        assert!(split_hunk(&mut diff_set, 0, 0));
        assert_eq!(diff_set.files[0].hunks.len(), 2);
        // First hunk: old a → new a
        assert_eq!(diff_set.files[0].hunks[0].lines.len(), 2);
        // Second hunk: old b → new b
        assert_eq!(diff_set.files[0].hunks[1].lines.len(), 2);
    }

    #[test]
    fn test_split_hunk_unsplittable() {
        // Single del/add pair — truly cannot split
        let mut diff_set = DiffSet::new("head".to_string(), "main".to_string(), "main".to_string());
        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);
        let mut hunk = Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1);
        hunk.lines = vec![
            DiffLine::new(LineOrigin::Deletion, "old\n".to_string(), Some(1), None),
            DiffLine::new(LineOrigin::Addition, "new\n".to_string(), None, Some(1)),
        ];
        file.hunks.push(hunk);
        diff_set.files.push(file);

        assert!(!split_hunk(&mut diff_set, 0, 0));
        assert_eq!(diff_set.files[0].hunks.len(), 1);
    }
}

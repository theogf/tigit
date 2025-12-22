/// Represents the complete diff between HEAD and main branch
#[derive(Debug, Clone)]
pub struct DiffSet {
    pub files: Vec<FileDiff>,
    #[allow(dead_code)]
    pub head_commit: String,
    #[allow(dead_code)]
    pub main_commit: String,
    pub main_branch_name: String,
}

impl DiffSet {
    pub fn new(head_commit: String, main_commit: String, main_branch_name: String) -> Self {
        Self {
            files: Vec::new(),
            head_commit,
            main_commit,
            main_branch_name,
        }
    }

    pub fn total_hunks(&self) -> usize {
        self.files.iter().map(|f| f.hunks.len()).sum()
    }

    pub fn selected_hunks(&self) -> usize {
        self.files
            .iter()
            .flat_map(|f| &f.hunks)
            .filter(|h| h.selected)
            .count()
    }

    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Diff for a single file
#[derive(Debug, Clone)]
pub struct FileDiff {
    pub path: String,
    pub status: FileStatus,
    pub hunks: Vec<Hunk>,
}

impl FileDiff {
    pub fn new(path: String, status: FileStatus) -> Self {
        Self {
            path,
            status,
            hunks: Vec::new(),
        }
    }

    #[allow(dead_code)]
    pub fn selected_hunks(&self) -> usize {
        self.hunks.iter().filter(|h| h.selected).count()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileStatus {
    Modified,
    Added,
    Deleted,
}

impl FileStatus {
    pub fn as_str(&self) -> &str {
        match self {
            FileStatus::Modified => "Modified",
            FileStatus::Added => "Added",
            FileStatus::Deleted => "Deleted",
        }
    }
}

/// A logical group of changed lines (diff hunk)
#[derive(Debug, Clone)]
pub struct Hunk {
    #[allow(dead_code)]
    pub id: usize,
    pub header: String,
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<DiffLine>,
    pub selected: bool,
}

impl Hunk {
    pub fn new(
        id: usize,
        header: String,
        old_start: u32,
        old_lines: u32,
        new_start: u32,
        new_lines: u32,
    ) -> Self {
        Self {
            id,
            header,
            old_start,
            old_lines,
            new_start,
            new_lines,
            lines: Vec::new(),
            selected: false,
        }
    }

    pub fn toggle_selection(&mut self) {
        self.selected = !self.selected;
    }
}

#[derive(Debug, Clone)]
pub struct DiffLine {
    pub origin: LineOrigin,
    pub content: String,
    pub old_lineno: Option<u32>,
    pub new_lineno: Option<u32>,
}

impl DiffLine {
    pub fn new(
        origin: LineOrigin,
        content: String,
        old_lineno: Option<u32>,
        new_lineno: Option<u32>,
    ) -> Self {
        Self {
            origin,
            content,
            old_lineno,
            new_lineno,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineOrigin {
    Context,
    Addition,
    Deletion,
}

impl LineOrigin {
    pub fn from_git2(origin: char) -> Self {
        match origin {
            '+' => LineOrigin::Addition,
            '-' => LineOrigin::Deletion,
            ' ' => LineOrigin::Context,
            _ => LineOrigin::Context,
        }
    }

    pub fn to_char(&self) -> char {
        match self {
            LineOrigin::Context => ' ',
            LineOrigin::Addition => '+',
            LineOrigin::Deletion => '-',
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_set_creation() {
        let diff_set = DiffSet::new(
            "abc123".to_string(),
            "def456".to_string(),
            "main".to_string(),
        );
        assert_eq!(diff_set.head_commit, "abc123");
        assert_eq!(diff_set.main_commit, "def456");
        assert_eq!(diff_set.main_branch_name, "main");
        assert!(diff_set.is_empty());
    }

    #[test]
    fn test_diff_set_total_hunks() {
        let mut diff_set = DiffSet::new(
            "abc123".to_string(),
            "def456".to_string(),
            "main".to_string(),
        );

        let mut file1 = FileDiff::new("file1.txt".to_string(), FileStatus::Modified);
        file1.hunks.push(Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1));
        file1.hunks.push(Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1));

        let mut file2 = FileDiff::new("file2.txt".to_string(), FileStatus::Modified);
        file2.hunks.push(Hunk::new(0, "@@ -10,1 +10,1 @@".to_string(), 10, 1, 10, 1));

        diff_set.files.push(file1);
        diff_set.files.push(file2);

        assert_eq!(diff_set.total_hunks(), 3);
        assert!(!diff_set.is_empty());
    }

    #[test]
    fn test_diff_set_selected_hunks() {
        let mut diff_set = DiffSet::new(
            "abc123".to_string(),
            "def456".to_string(),
            "main".to_string(),
        );

        let mut file = FileDiff::new("file.txt".to_string(), FileStatus::Modified);
        let mut hunk1 = Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1);
        let mut hunk2 = Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1);
        let hunk3 = Hunk::new(2, "@@ -10,1 +10,1 @@".to_string(), 10, 1, 10, 1);

        hunk1.selected = true;
        hunk2.selected = true;

        file.hunks.push(hunk1);
        file.hunks.push(hunk2);
        file.hunks.push(hunk3);

        diff_set.files.push(file);

        assert_eq!(diff_set.selected_hunks(), 2);
    }

    #[test]
    fn test_file_diff_creation() {
        let file = FileDiff::new("test.rs".to_string(), FileStatus::Modified);
        assert_eq!(file.path, "test.rs");
        assert_eq!(file.status, FileStatus::Modified);
        assert!(file.hunks.is_empty());
    }

    #[test]
    fn test_file_status_as_str() {
        assert_eq!(FileStatus::Modified.as_str(), "Modified");
        assert_eq!(FileStatus::Added.as_str(), "Added");
        assert_eq!(FileStatus::Deleted.as_str(), "Deleted");
    }

    #[test]
    fn test_hunk_toggle_selection() {
        let mut hunk = Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1);
        assert!(!hunk.selected);

        hunk.toggle_selection();
        assert!(hunk.selected);

        hunk.toggle_selection();
        assert!(!hunk.selected);
    }

    #[test]
    fn test_line_origin_from_git2() {
        assert_eq!(LineOrigin::from_git2('+'), LineOrigin::Addition);
        assert_eq!(LineOrigin::from_git2('-'), LineOrigin::Deletion);
        assert_eq!(LineOrigin::from_git2(' '), LineOrigin::Context);
        assert_eq!(LineOrigin::from_git2('?'), LineOrigin::Context); // Unknown defaults to context
    }

    #[test]
    fn test_line_origin_to_char() {
        assert_eq!(LineOrigin::Addition.to_char(), '+');
        assert_eq!(LineOrigin::Deletion.to_char(), '-');
        assert_eq!(LineOrigin::Context.to_char(), ' ');
    }

    #[test]
    fn test_diff_line_creation() {
        let line = DiffLine::new(
            LineOrigin::Addition,
            "println!(\"hello\");\n".to_string(),
            None,
            Some(10),
        );

        assert_eq!(line.origin, LineOrigin::Addition);
        assert_eq!(line.content, "println!(\"hello\");\n");
        assert_eq!(line.old_lineno, None);
        assert_eq!(line.new_lineno, Some(10));
    }

    #[test]
    fn test_file_diff_selected_hunks() {
        let mut file = FileDiff::new("test.txt".to_string(), FileStatus::Modified);

        let mut hunk1 = Hunk::new(0, "@@ -1,1 +1,1 @@".to_string(), 1, 1, 1, 1);
        hunk1.selected = true;

        let hunk2 = Hunk::new(1, "@@ -5,1 +5,1 @@".to_string(), 5, 1, 5, 1);

        file.hunks.push(hunk1);
        file.hunks.push(hunk2);

        assert_eq!(file.selected_hunks(), 1);
    }
}

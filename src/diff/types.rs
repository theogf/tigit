/// Represents the complete diff between HEAD and main branch
#[derive(Debug, Clone)]
pub struct DiffSet {
    pub files: Vec<FileDiff>,
    pub head_commit: String,
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

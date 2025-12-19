// Parser utilities for diff hunks
// Most parsing is done in git/diff.rs using git2 callbacks
// This module can contain additional parsing helpers if needed

use crate::error::{GitDiffError, Result};
use regex::Regex;
use std::sync::OnceLock;

/// Parse hunk header line like "@@ -10,5 +12,7 @@ function_name"
pub fn parse_hunk_header(header: &str) -> Result<(u32, u32, u32, u32)> {
    static HUNK_REGEX: OnceLock<Regex> = OnceLock::new();
    let re = HUNK_REGEX.get_or_init(|| {
        Regex::new(r"@@ -(\d+)(?:,(\d+))? \+(\d+)(?:,(\d+))? @@").unwrap()
    });

    if let Some(caps) = re.captures(header) {
        let old_start = caps.get(1).unwrap().as_str().parse::<u32>().unwrap();
        let old_lines = caps
            .get(2)
            .map(|m| m.as_str().parse::<u32>().unwrap())
            .unwrap_or(1);
        let new_start = caps.get(3).unwrap().as_str().parse::<u32>().unwrap();
        let new_lines = caps
            .get(4)
            .map(|m| m.as_str().parse::<u32>().unwrap())
            .unwrap_or(1);

        Ok((old_start, old_lines, new_start, new_lines))
    } else {
        Err(GitDiffError::InvalidDiff(format!(
            "Invalid hunk header: {}",
            header
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_hunk_header() {
        let header = "@@ -10,5 +12,7 @@ fn main()";
        let (old_start, old_lines, new_start, new_lines) = parse_hunk_header(header).unwrap();
        assert_eq!(old_start, 10);
        assert_eq!(old_lines, 5);
        assert_eq!(new_start, 12);
        assert_eq!(new_lines, 7);
    }

    #[test]
    fn test_parse_hunk_header_no_count() {
        let header = "@@ -10 +12 @@ fn main()";
        let (old_start, old_lines, new_start, new_lines) = parse_hunk_header(header).unwrap();
        assert_eq!(old_start, 10);
        assert_eq!(old_lines, 1);
        assert_eq!(new_start, 12);
        assert_eq!(new_lines, 1);
    }
}

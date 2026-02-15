[![CI](https://github.com/theogf/tigit/actions/workflows/rust.yml/badge.svg)](https://github.com/theogf/tigit/actions/workflows/rust.yml)

# tigit

An interactive TUI (Terminal User Interface) for comparing your working directory with the main branch and selectively reverting changes (both committed and uncommitted).

> [!WARNING]
> This project was vibecoded with [Claude](https://claude.ai). While functional and tested, use at your own risk.

## Features

- Side-by-side diff view comparing working directory and main branch
- Shows both committed and uncommitted changes
- Interactive navigation through files and hunks
- Select individual hunks to revert
- Stage reverted changes with git
- Auto-detects main branch (main/master)
- Built with Rust and ratatui

## Installation

```bash
cargo install --git https://github.com/theogf/tigit
```

(make sure that the `.cargo/bin` directory is in your `PATH`).

## Usage

```bash
# Specify a different repository path
tigit --path /path/to/repo

# Specify a custom branch to compare against (`main` by default)
tigit --branch develop

# Allow running even with uncommitted changes
tigit --allow-dirty
```

## Keyboard Shortcuts

### Navigation
- `↑/k` - Previous hunk
- `↓/j` - Next hunk
- `Shift+↑/↓` - Scroll vertically within hunk
- `Shift+←/→` - Scroll horizontally within hunk (for long lines)
- `Tab` - Next file
- `Shift+Tab` - Previous file
- `PgUp/PgDn` - Scroll by page
- `Home/End` - First/last hunk

### Selection
- `Space` - Toggle current hunk selection
- `a` - Select all hunks in current file
- `n` - Deselect all hunks in current file
- `A` - Select all hunks globally (across all files)
- `N` - Deselect all hunks globally

### Actions
- `Enter` - Apply selected reversions (shows confirmation dialog)
- `r` - Refresh diff from git
- `?` - Toggle help panel
- `q/Esc` - Quit

## How It Works

### Merge-Base Diff (main...working directory)

The tool compares your **working directory** (including uncommitted changes) against the merge base with main:
```bash
# Shows changes from merge base to working directory
git diff $(git merge-base main HEAD)
```

**Why this matters:**
- Shows only **your branch's changes**, excluding main's evolution
- Includes both **committed and uncommitted** changes
- Standard for pull request reviews, but with uncommitted changes visible too
- Example: If main moved from A→B→C and you branched at B creating D→E, shows D→E plus any uncommitted edits

### What Gets Staged

When you press Enter:
- The selected hunks are **reversed** (their changes are undone)
- The reversed changes are **automatically staged** (added to git index)
- Your working directory files are modified to match the main branch for those hunks
- Other hunks (not selected) remain unchanged

## Limitations

- Does not show completely untracked files (only tracked files with changes)

## License

This project is open source.

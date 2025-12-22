[![CI](https://github.com/theogf/tigit/actions/workflows/rust.yml/badge.svg)](https://github.com/theogf/tigit/actions/workflows/rust.yml)

# tigit

An interactive TUI (Terminal User Interface) for comparing git HEAD with the main branch and selectively reverting changes.

> [!WARNING]
> This project was vibecoded with [Claude](https://claude.ai). While functional and tested, use at your own risk.

## Features

- Side-by-side diff view comparing HEAD and main branch
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
- `Tab` - Next file
- `Shift+Tab` - Previous file
- `PgUp/PgDn` - Scroll by page
- `Home/End` - First/last hunk

### Selection
- `Space` - Toggle current hunk selection
- `a` - Select all hunks in current file
- `n` - Deselect all hunks in current file

### Actions
- `Enter` - Apply selected reversions (stage with git)
- `r` - Refresh diff (not yet implemented)
- `?` - Toggle help panel
- `q/Esc` - Quit

## How It Works

### Merge-Base Diff (main...HEAD)

The tool uses **three-dot notation** (`main...HEAD`) showing only changes on your branch since diverging from main:
```bash
# Equivalent to:
git diff $(git merge-base main HEAD)..HEAD
```

**Why this matters:**
- Shows only **your branch's changes**, excluding main's evolution
- Standard for pull request reviews
- Example: If main moved from A→B→C and you branched at B creating D→E, only shows D→E

### What Gets Staged

When you press Enter:
- The selected hunks are **reversed** (their changes are undone)
- The reversed changes are **automatically staged** (added to git index)
- Your working directory files are modified to match the main branch for those hunks
- Other hunks (not selected) remain unchanged

## Limitations

- Compares commits (HEAD vs main), not working directory changes

## License

This project is open source.

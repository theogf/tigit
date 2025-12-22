# tigit

An interactive TUI (Terminal User Interface) for comparing git HEAD with the main branch and selectively reverting changes.

## Features

- Side-by-side diff view comparing HEAD and main branch
- Interactive navigation through files and hunks
- Select individual hunks to revert
- Stage reverted changes with git
- Auto-detects main branch (main/master)
- Built with Rust and ratatui

## Installation

```bash
cargo build --release
```

The binary will be available at `target/release/tigit`.

## Usage

```bash
# Run in current directory (auto-detects main branch)
cargo run

# Or use the binary
./target/release/tigit

# Specify a different repository path
tigit --path /path/to/repo

# Specify a custom branch to compare against
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

### Understanding the Revert Feature

**What is a "revert"?**
Reverting a hunk means **undoing** specific changes from your current branch by restoring the merge-base version. Useful when you've made multiple changes but want to keep only some.

**Example:**
- Merge base has: `version = "1.0"`
- Your HEAD has: `version = "2.0"` (you changed it)
- **Reverting this hunk** restores it back to: `version = "1.0"`

### Step-by-Step Process

1. **Find merge base:** Finds common ancestor between main and HEAD
2. **Compare commits:** Shows changes from merge base to your current HEAD
2. **View differences:** Shows side-by-side:
   - **Left column:** Main branch version (what you had before)
   - **Right column:** HEAD version (what you have now)
   - **Green background:** Selected hunks that will be reverted
3. **Select hunks:** Use `Space` to mark hunks for reverting
   - Selected hunks will have their changes **undone** (restored to main branch version)
4. **Apply reversions:** Press `Enter` to:
   - Generate a reverse patch that undoes the selected changes
   - Apply the patch to your working directory
   - **Stage the reversions** with git (ready to commit)
5. **Review:** Check staged changes with `git diff --cached`
6. **Commit:** If satisfied, commit the reversions

### What Gets Staged

When you press Enter:
- The selected hunks are **reversed** (their changes are undone)
- The reversed changes are **automatically staged** (added to git index)
- Your working directory files are modified to match the main branch for those hunks
- Other hunks (not selected) remain unchanged

## Example Workflow

```bash
# Make some changes on a feature branch
git checkout -b feature/my-feature
# ... make changes ...

# Run tigit to selectively revert some changes
tigit

# In the TUI:
# - Navigate with ↑/↓
# - Press Space to select hunks to revert
# - Press Enter to apply

# Review what was staged
git diff --cached

# Commit if satisfied
git commit -m "Revert selected changes"
```

## Architecture

```
src/
├── main.rs              # Entry point, CLI args, terminal setup
├── error.rs             # Custom error types
├── git/
│   ├── repository.rs    # Open repo, detect main branch
│   ├── diff.rs          # Extract diff using git2
│   └── revert.rs        # Generate reverse patch, stage changes
├── diff/
│   ├── types.rs         # DiffSet, FileDiff, Hunk, DiffLine
│   ├── parser.rs        # Parse diff headers
│   └── selection.rs     # Track selected hunks
└── tui/
    ├── app.rs           # App state, event handling
    ├── ui.rs            # Render side-by-side layout
    ├── events.rs        # Input event loop
    └── widgets/
        ├── diff_view.rs # Side-by-side diff widget
        └── help.rs      # Help panel
```

## Dependencies

- `ratatui` - Modern TUI framework
- `crossterm` - Cross-platform terminal manipulation
- `git2` - Git operations via libgit2
- `clap` - CLI argument parsing
- `anyhow` - Error handling
- `thiserror` - Custom error types
- `regex` - Pattern matching
- `unicode-width` - Text alignment

## Testing

The project includes a comprehensive test suite with **58 tests**:
- **50 unit tests** covering core functionality (data structures, selection, parsing, reverse patch generation)
- **8 integration tests** using temporary git repositories to test real git operations

Run tests with:
```bash
cargo test
```

See [TESTING.md](TESTING.md) for detailed testing documentation.

## Limitations

- Compares commits (HEAD vs main), not working directory changes
- Requires clean working directory (or use `--allow-dirty`)
- Binary files are not supported
- Large diffs may require scrolling

## License

This project is open source.

# git-main-diff

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

The binary will be available at `target/release/git-main-diff`.

## Usage

```bash
# Run in current directory (auto-detects main branch)
cargo run

# Or use the binary
./target/release/git-main-diff

# Specify a different repository path
git-main-diff --path /path/to/repo

# Specify a custom branch to compare against
git-main-diff --branch develop

# Allow running even with uncommitted changes
git-main-diff --allow-dirty
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

1. Compares the current git HEAD commit to the main branch
2. Displays differences in a side-by-side view:
   - Left column: Main branch version
   - Right column: HEAD version
3. Select hunks you want to revert (restore main branch version)
4. Press Enter to apply reversions and stage them with git
5. Review staged changes with `git diff --cached`
6. Commit the changes if desired

## Example Workflow

```bash
# Make some changes on a feature branch
git checkout -b feature/my-feature
# ... make changes ...

# Run git-main-diff to selectively revert some changes
git-main-diff

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

## Limitations

- Compares commits (HEAD vs main), not working directory changes
- Requires clean working directory (or use `--allow-dirty`)
- Binary files are not supported
- Large diffs may require scrolling

## License

This project is open source.

# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

tigit is an interactive TUI (Terminal User Interface) for comparing the working directory (including uncommitted changes) with the main branch and selectively reverting changes. Built with Rust using ratatui for the terminal interface and git2 for git operations.

## Build and Development Commands

```bash
# Build
cargo build

# Run (from any directory)
cargo run -- --path /path/to/repo

# Run tests
cargo test

# Run a single test
cargo test test_name

# Run tests with output
cargo test -- --nocapture

# Check formatting
cargo fmt --check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Architecture

### Module Structure

- **`src/main.rs`** - Entry point, CLI parsing (clap), terminal setup/teardown, main event loop
- **`src/lib.rs`** - Library exports for integration tests (`diff`, `error`, `git` modules)

### Core Modules

**`src/git/`** - Git operations via git2 library
- `repository.rs` - Open repo, detect main branch, get commits
- `diff.rs` - Extract diff using merge-base comparing to working directory (includes uncommitted changes)
- `revert.rs` - Generate reverse patches and apply via `git apply --index`

**`src/diff/`** - Diff data structures and manipulation
- `types.rs` - `DiffSet`, `FileDiff`, `Hunk`, `DiffLine` structs
- `parser.rs` - Diff parsing utilities
- `selection.rs` - Hunk selection logic

**`src/tui/`** - Terminal UI (ratatui-based)
- `app.rs` - Application state (`App` struct), keyboard event handling, navigation
- `ui.rs` - Main render function
- `events.rs` - Keyboard event reading
- `widgets/` - UI components (diff_view, help panel)

### Key Data Flow

1. `git::diff::extract_diff_set()` extracts diff from merge-base to working directory (includes uncommitted changes)
2. Diff is parsed into `DiffSet` containing `FileDiff`s with `Hunk`s
3. `tui::app::App` manages state (current file/hunk, selections, scroll positions)
4. User selects hunks, confirms with Enter (shows confirmation dialog)
5. `git::revert::create_reverse_patch()` generates inverted patch
6. Patch applied via `git apply --index` to working directory and staging area

### Error Handling

Custom `GitDiffError` enum in `src/error.rs` using `thiserror`. All fallible functions return `Result<T>` alias.

## Testing

Integration tests in `tests/integration_test.rs` create temporary git repositories using `tempfile` crate to test diff extraction and patch generation.

## Rust Edition

Uses Rust 2024 edition (`edition = "2024"` in Cargo.toml).

# Testing Documentation

## Test Summary

The project includes a comprehensive test suite with **58 passing tests** covering unit and integration testing.

### Test Categories

#### Unit Tests (50 tests total)

**diff/types.rs (10 tests)**
- `test_diff_set_creation` - DiffSet initialization
- `test_diff_set_total_hunks` - Hunk counting across files
- `test_diff_set_selected_hunks` - Selected hunk counting
- `test_file_diff_creation` - FileDiff initialization
- `test_file_status_as_str` - FileStatus string conversion
- `test_hunk_toggle_selection` - Hunk selection toggling
- `test_line_origin_from_git2` - LineOrigin from git2 char conversion
- `test_line_origin_to_char` - LineOrigin to char conversion
- `test_diff_line_creation` - DiffLine initialization
- `test_file_diff_selected_hunks` - Selected hunks in a file

**diff/selection.rs (7 tests)**
- `test_toggle_hunk_selection` - Toggle hunk selection in DiffSet
- `test_toggle_invalid_indices` - Handle invalid indices gracefully
- `test_select_all_in_file` - Select all hunks in a file
- `test_deselect_all_in_file` - Deselect all hunks in a file
- `test_get_selected_hunks` - Get indices of selected hunks
- `test_get_selected_hunks_empty` - Handle empty selection

**diff/parser.rs (2 tests)**
- `test_parse_hunk_header` - Parse standard hunk headers
- `test_parse_hunk_header_no_count` - Parse hunk headers without line counts

**git/revert.rs (8 tests)**
- `test_create_reverse_patch_empty` - Empty patch for no selections
- `test_create_reverse_patch_no_selection` - No patch when nothing selected
- `test_create_reverse_patch_simple_addition` - Reverse a simple addition
- `test_create_reverse_patch_simple_deletion` - Reverse a simple deletion
- `test_create_reverse_patch_multiple_hunks` - Multiple hunk selection
- `test_create_reverse_patch_multiple_files` - Multiple file handling
- `test_reverse_patch_line_without_newline` - Handle lines without newlines

#### Integration Tests (8 tests)

**tests/integration_test.rs**
- `test_repository_opening` - Open and discover git repositories
- `test_detect_main_branch_with_main` - Auto-detect main branch
- `test_extract_diff_simple` - Extract simple file modifications
- `test_diff_with_multiple_files` - Handle multiple modified files
- `test_diff_with_new_file` - Detect newly added files
- `test_diff_with_deleted_file` - Detect deleted files
- `test_reverse_patch_format` - Verify reverse patch format
- `test_is_working_directory_clean` - Check working directory status

## Running Tests

### Run all tests
```bash
cargo test
```

### Run specific test suite
```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_test

# Specific module tests
cargo test diff::types::tests
cargo test git::revert::tests
```

### Run tests with output
```bash
cargo test -- --nocapture
```

### Run tests with backtrace
```bash
RUST_BACKTRACE=1 cargo test
```

## Test Coverage

### Core Functionality
- ✅ Data structures (DiffSet, FileDiff, Hunk, DiffLine)
- ✅ Selection management
- ✅ Diff parsing
- ✅ Reverse patch generation
- ✅ Git repository operations
- ✅ Branch detection
- ✅ Diff extraction

### Edge Cases Tested
- Empty diff sets
- No selections
- Invalid indices
- Lines without newlines
- Multiple files and hunks
- Added, modified, and deleted files
- Clean vs dirty working directories

## Integration Test Approach

Integration tests create temporary git repositories using `tempfile::TempDir` to:
1. Test real git operations without affecting the local system
2. Verify end-to-end workflows
3. Ensure git2 library integration works correctly

### Key Testing Pattern

```rust
// Create temporary repo
let (_temp_dir, repo) = create_test_repo();

// Create initial commit (main branch)
fs::write(path, "content").unwrap();
let main_commit_id = create_commit(&repo, "Initial");

// Detach HEAD to prevent branch movement
repo.set_head_detached(main_commit_id).unwrap();

// Make changes and commit (HEAD advances, main stays)
fs::write(path, "modified").unwrap();
create_commit(&repo, "Changes");

// Extract and verify diff
let diff_set = extract_diff_set(&repo, "main").unwrap();
```

## Future Test Additions

Potential areas for additional testing:
- [ ] TUI rendering (visual snapshot tests)
- [ ] Keyboard event handling
- [ ] Error recovery scenarios
- [ ] Binary file handling
- [ ] Large diff performance
- [ ] Unicode and special characters
- [ ] Concurrent access scenarios

## Continuous Integration

To add CI/CD testing, create `.github/workflows/test.yml`:

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo test --all-features
```

## Test Maintenance

- Keep tests focused and isolated
- Update tests when adding new features
- Ensure tests are deterministic (no race conditions)
- Use descriptive test names
- Document complex test scenarios

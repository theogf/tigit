use git2::{Repository, Signature};
use std::fs;
use tempfile::TempDir;

// Helper function to create a test git repository
fn create_test_repo() -> (TempDir, Repository) {
    let temp_dir = TempDir::new().unwrap();
    let repo = Repository::init(temp_dir.path()).unwrap();

    // Configure repo
    let mut config = repo.config().unwrap();
    config.set_str("user.name", "Test User").unwrap();
    config.set_str("user.email", "test@example.com").unwrap();

    (temp_dir, repo)
}

// Helper function to create a commit
fn create_commit(repo: &Repository, message: &str) -> git2::Oid {
    let mut index = repo.index().unwrap();
    index.add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();

    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();

    let signature = Signature::now("Test User", "test@example.com").unwrap();

    let parent_commit = repo.head().ok().and_then(|h| h.peel_to_commit().ok());

    let parents = if let Some(ref p) = parent_commit {
        vec![p]
    } else {
        vec![]
    };

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        message,
        &tree,
        &parents,
    ).unwrap()
}

#[test]
fn test_repository_opening() {
    let (_temp_dir, repo) = create_test_repo();

    // Test that we can open the repository
    let opened = git_main_diff::git::repository::open_repository(Some(repo.path().parent().unwrap())).unwrap();
    assert!(!opened.is_bare());
}

#[test]
fn test_detect_main_branch_with_main() {
    let (_temp_dir, repo) = create_test_repo();

    // Create initial commit on main
    let repo_path = repo.path().parent().unwrap();
    fs::write(repo_path.join("file.txt"), "initial content").unwrap();
    create_commit(&repo, "Initial commit");

    // Should detect "main" branch (modern default)
    let branch = git_main_diff::git::repository::detect_main_branch(&repo);
    // This might be "main" or "master" depending on git config
    assert!(branch.is_ok());
}

#[test]
fn test_extract_diff_simple() {
    let (_temp_dir, repo) = create_test_repo();
    let repo_path = repo.path().parent().unwrap();

    // Create initial file and commit on main
    fs::write(repo_path.join("file.txt"), "line 1\nline 2\nline 3\n").unwrap();
    let main_commit_id = create_commit(&repo, "Initial commit on main");
    let main_commit = repo.find_commit(main_commit_id).unwrap();

    // Ensure main branch exists and points to this commit
    let branch_name = "main";
    if repo.find_branch(branch_name, git2::BranchType::Local).is_err() {
        repo.branch(branch_name, &main_commit, false).unwrap();
    }

    // Detach HEAD to make changes without moving the main branch
    repo.set_head_detached(main_commit_id).unwrap();

    // Make changes and commit (HEAD advances, main stays)
    fs::write(repo_path.join("file.txt"), "line 1\nmodified line 2\nline 3\nnew line 4\n").unwrap();
    create_commit(&repo, "Modify file");

    // Extract diff between HEAD and main
    let diff_set = git_main_diff::git::diff::extract_diff_set(&repo, branch_name);

    assert!(diff_set.is_ok());
    let diff_set = diff_set.unwrap();

    assert_eq!(diff_set.files.len(), 1);
    assert_eq!(diff_set.files[0].path, "file.txt");
    assert!(!diff_set.files[0].hunks.is_empty());
}

#[test]
fn test_diff_with_multiple_files() {
    let (_temp_dir, repo) = create_test_repo();
    let repo_path = repo.path().parent().unwrap();

    // Create main branch with multiple files
    fs::write(repo_path.join("file1.txt"), "content 1\n").unwrap();
    fs::write(repo_path.join("file2.txt"), "content 2\n").unwrap();
    let main_commit_id = create_commit(&repo, "Initial commit");
    let main_commit = repo.find_commit(main_commit_id).unwrap();

    // Ensure main branch exists
    let branch_name = "main";
    if repo.find_branch(branch_name, git2::BranchType::Local).is_err() {
        repo.branch(branch_name, &main_commit, false).unwrap();
    }

    // Detach HEAD
    repo.set_head_detached(main_commit_id).unwrap();

    // Modify both files
    fs::write(repo_path.join("file1.txt"), "modified content 1\n").unwrap();
    fs::write(repo_path.join("file2.txt"), "modified content 2\n").unwrap();
    create_commit(&repo, "Modify both files");

    // Extract diff
    let diff_set = git_main_diff::git::diff::extract_diff_set(&repo, branch_name).unwrap();

    assert_eq!(diff_set.files.len(), 2);
}

#[test]
fn test_diff_with_new_file() {
    let (_temp_dir, repo) = create_test_repo();
    let repo_path = repo.path().parent().unwrap();

    // Create main branch
    fs::write(repo_path.join("existing.txt"), "content\n").unwrap();
    let main_commit_id = create_commit(&repo, "Initial commit");
    let main_commit = repo.find_commit(main_commit_id).unwrap();

    // Ensure main branch exists
    let branch_name = "main";
    if repo.find_branch(branch_name, git2::BranchType::Local).is_err() {
        repo.branch(branch_name, &main_commit, false).unwrap();
    }

    // Detach HEAD
    repo.set_head_detached(main_commit_id).unwrap();

    // Add new file
    fs::write(repo_path.join("new.txt"), "new content\n").unwrap();
    create_commit(&repo, "Add new file");

    // Extract diff
    let diff_set = git_main_diff::git::diff::extract_diff_set(&repo, branch_name).unwrap();

    // Should show the new file
    let new_file = diff_set.files.iter().find(|f| f.path == "new.txt");
    assert!(new_file.is_some());
    assert_eq!(new_file.unwrap().status, git_main_diff::diff::types::FileStatus::Added);
}

#[test]
fn test_diff_with_deleted_file() {
    let (_temp_dir, repo) = create_test_repo();
    let repo_path = repo.path().parent().unwrap();

    // Create main branch with file to delete
    fs::write(repo_path.join("to_delete.txt"), "content\n").unwrap();
    let main_commit_id = create_commit(&repo, "Initial commit");
    let main_commit = repo.find_commit(main_commit_id).unwrap();

    // Ensure main branch exists
    let branch_name = "main";
    if repo.find_branch(branch_name, git2::BranchType::Local).is_err() {
        repo.branch(branch_name, &main_commit, false).unwrap();
    }

    // Detach HEAD
    repo.set_head_detached(main_commit_id).unwrap();

    // Delete the file
    fs::remove_file(repo_path.join("to_delete.txt")).unwrap();
    create_commit(&repo, "Delete file");

    // Extract diff
    let diff_set = git_main_diff::git::diff::extract_diff_set(&repo, branch_name).unwrap();

    // Should show the deleted file
    let deleted_file = diff_set.files.iter().find(|f| f.path == "to_delete.txt");
    assert!(deleted_file.is_some());
    assert_eq!(deleted_file.unwrap().status, git_main_diff::diff::types::FileStatus::Deleted);
}

#[test]
fn test_reverse_patch_format() {
    use git_main_diff::diff::types::{DiffSet, FileDiff, FileStatus, Hunk, DiffLine, LineOrigin};
    use git_main_diff::git::revert::create_reverse_patch;

    let mut diff_set = DiffSet::new(
        "head123".to_string(),
        "main456".to_string(),
        "main".to_string(),
    );

    let mut file = FileDiff::new("test.rs".to_string(), FileStatus::Modified);
    let mut hunk = Hunk::new(0, "@@ -5,3 +5,4 @@ fn test()".to_string(), 5, 3, 5, 4);
    hunk.selected = true;

    hunk.lines.push(DiffLine::new(
        LineOrigin::Context,
        "    let x = 5;\n".to_string(),
        Some(5),
        Some(5),
    ));
    hunk.lines.push(DiffLine::new(
        LineOrigin::Addition,
        "    let y = 10;\n".to_string(),
        None,
        Some(6),
    ));
    hunk.lines.push(DiffLine::new(
        LineOrigin::Context,
        "    println!(\"{}\", x);\n".to_string(),
        Some(6),
        Some(7),
    ));

    file.hunks.push(hunk);
    diff_set.files.push(file);

    let patch = create_reverse_patch(&diff_set).unwrap();

    // Verify patch format
    assert!(patch.starts_with("diff --git"));
    assert!(patch.contains("--- a/test.rs"));
    assert!(patch.contains("+++ b/test.rs"));
    assert!(patch.contains("@@ -5,4 +5,3 @@")); // Reversed ranges
    assert!(patch.contains(" ")); // Context line
    assert!(patch.contains("-    let y = 10;")); // Reversed addition becomes deletion
}

#[test]
fn test_is_working_directory_clean() {
    let (_temp_dir, repo) = create_test_repo();
    let repo_path = repo.path().parent().unwrap();

    // Initially clean (no commits or files)
    assert!(git_main_diff::git::repository::is_working_directory_clean(&repo).unwrap());

    // Add a file but don't commit
    fs::write(repo_path.join("uncommitted.txt"), "content\n").unwrap();

    // Should not be clean
    assert!(!git_main_diff::git::repository::is_working_directory_clean(&repo).unwrap());

    // Commit the file
    create_commit(&repo, "Commit file");

    // Should be clean again
    assert!(git_main_diff::git::repository::is_working_directory_clean(&repo).unwrap());
}

use giff_git::{GitBackend, RebaseOutcome, ShellGitBackend};
use std::process::Command;
use tempfile::TempDir;

fn init_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    Command::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    // initial commit so main exists
    std::fs::write(dir.path().join("README.md"), "init").unwrap();
    Command::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output()
        .unwrap();
    Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    dir
}

#[test]
fn current_branch_returns_main() {
    let dir = init_repo();
    let backend = ShellGitBackend::new(dir.path().to_path_buf());
    assert_eq!(backend.current_branch().unwrap(), "main");
}

#[test]
fn create_and_checkout_branch() {
    let dir = init_repo();
    let backend = ShellGitBackend::new(dir.path().to_path_buf());
    backend.create_branch("feat/test", "main").unwrap();
    backend.checkout("feat/test").unwrap();
    assert_eq!(backend.current_branch().unwrap(), "feat/test");
}

#[test]
fn branch_exists_returns_true_for_main() {
    let dir = init_repo();
    let backend = ShellGitBackend::new(dir.path().to_path_buf());
    assert!(backend.branch_exists("main").unwrap());
    assert!(!backend.branch_exists("nonexistent").unwrap());
}

#[test]
fn merge_base_returns_sha() {
    let dir = init_repo();
    let backend = ShellGitBackend::new(dir.path().to_path_buf());
    backend.create_branch("feat/a", "main").unwrap();
    // merge-base of main with itself is HEAD
    let base = backend.merge_base("main", "main").unwrap();
    assert!(!base.is_empty());
}

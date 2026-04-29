use assert_cmd::Command;
use std::process::Command as StdCommand;
use tempfile::TempDir;

fn init_git_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    StdCommand::new("git")
        .args(["init", "-b", "main"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["config", "user.email", "t@t.com"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["config", "user.name", "T"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    std::fs::write(dir.path().join("f"), "x").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(dir.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(dir.path())
        .output()
        .unwrap();
    dir
}

fn current_branch(dir: &std::path::Path) -> String {
    let out = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn git_commit_file(dir: &std::path::Path, filename: &str, msg: &str) {
    std::fs::write(dir.join(filename), msg).unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", msg])
        .current_dir(dir)
        .output()
        .unwrap();
}

#[test]
fn giff_help_exits_zero() {
    Command::cargo_bin("giff")
        .unwrap()
        .arg("--help")
        .assert()
        .success();
}

#[test]
fn giff_init_creates_config_skeleton() {
    let dir = tempfile::TempDir::new().unwrap();
    Command::cargo_bin("giff")
        .unwrap()
        .env("HOME", dir.path())
        .arg("init")
        .assert()
        .success();
}

#[test]
fn giff_new_creates_branch_and_frame() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/my-feature"])
        .assert()
        .success();

    // Branch should exist
    let out = StdCommand::new("git")
        .args(["branch", "--list", "feat/my-feature"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert!(String::from_utf8_lossy(&out.stdout).contains("feat/my-feature"));

    // stacked.toml should exist inside .git/
    assert!(repo.path().join(".git").join("stacked.toml").exists());
}

#[test]
fn giff_log_shows_stack() {
    let repo = init_git_repo();
    // Create a stack
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();

    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("log")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("feat/a"));
    assert!(stdout.contains("feat/b"));
}

#[test]
fn giff_status_shows_current_branch() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/x"])
        .assert()
        .success();

    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("status")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("feat/x"));
}

#[test]
fn giff_next_and_prev_navigate_stack() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    // currently on feat/b — go prev
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("prev")
        .assert()
        .success();
    let out = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "feat/a");
    // go next
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("next")
        .assert()
        .success();
    let out = StdCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "feat/b");
}

#[test]
fn giff_push_requires_token() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/push-test"])
        .assert()
        .success();
    // Without a token and remote, push should fail with a meaningful error (not a panic).
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .env("GITHUB_TOKEN", "")
        .arg("push")
        .output()
        .unwrap();
    // Should exit non-zero with an error message
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("error"));
}

#[test]
fn giff_sync_restacks_clean_stack() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    // Add a commit on feat/a
    std::fs::write(repo.path().join("feat_a.txt"), "content").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(repo.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "feat a"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    // Add a commit on main (simulate upstream update)
    StdCommand::new("git")
        .args(["checkout", "main"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    std::fs::write(repo.path().join("main_update.txt"), "upstream").unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(repo.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["commit", "-m", "upstream"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    StdCommand::new("git")
        .args(["checkout", "feat/a"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    // sync should restack without conflict
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("sync")
        .assert()
        .success();
}

#[test]
fn giff_stack_drop_removes_frame() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["stack", "drop", "feat/b"])
        .assert()
        .success();
    // .git/stacked.toml should no longer contain feat/b
    let toml = std::fs::read_to_string(repo.path().join(".git/stacked.toml")).unwrap();
    assert!(!toml.contains("feat/b"));
}

#[test]
fn giff_new_uses_config_trunk() {
    let home = TempDir::new().unwrap();
    // dirs::config_dir() is platform-specific; write to both common locations.
    for subpath in &["Library/Application Support/giff", ".config/giff"] {
        let cfg_dir = home.path().join(subpath);
        std::fs::create_dir_all(&cfg_dir).unwrap();
        std::fs::write(
            cfg_dir.join("config.toml"),
            "[defaults]\ntrunk = \"develop\"\n",
        )
        .unwrap();
    }

    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .env("HOME", home.path())
        .args(["new", "feat/custom-trunk"])
        .assert()
        .success();

    let toml = std::fs::read_to_string(repo.path().join(".git/stacked.toml")).unwrap();
    assert!(
        toml.contains("develop"),
        "expected trunk = develop in stacked.toml, got:\n{}",
        toml
    );
}

#[test]
fn giff_checkout_by_position() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    // Currently on feat/b (position 2). Checkout position 1 (feat/a).
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["checkout", "1"])
        .assert()
        .success();
    assert_eq!(current_branch(repo.path()), "feat/a");
}

#[test]
fn giff_stack_squash_merges_frame_below() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    git_commit_file(repo.path(), "a.txt", "commit on feat/a");

    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    git_commit_file(repo.path(), "b.txt", "commit on feat/b");

    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["stack", "squash", "feat/b"])
        .assert()
        .success();

    // feat/b should be gone from stacked.toml
    let toml = std::fs::read_to_string(repo.path().join(".git/stacked.toml")).unwrap();
    assert!(
        !toml.contains("feat/b"),
        "feat/b should be removed after squash"
    );

    // We should be on feat/a now
    assert_eq!(current_branch(repo.path()), "feat/a");
}

#[test]
fn giff_sync_continue_errors_without_resume_state() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    // --continue with no saved state should fail with a clear message
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["sync", "--continue"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no interrupted sync"),
        "expected 'no interrupted sync' in stderr, got: {}",
        stderr
    );
}

#[test]
fn giff_sync_errors_when_resume_state_exists() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();

    // Manually plant a resume state file
    let resume = r#"{"stack_id":"s1","resume_from_idx":1,"original_branch":"feat/a"}"#;
    std::fs::write(repo.path().join(".git/giff_sync_resume.json"), resume).unwrap();

    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("sync")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("interrupted"),
        "expected 'interrupted' in stderr, got: {}",
        stderr
    );
}

#[test]
fn giff_drop_relinks_parent_of_frame_above() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/c"])
        .assert()
        .success();

    // Drop the middle frame
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["stack", "drop", "feat/b"])
        .assert()
        .success();

    let toml = std::fs::read_to_string(repo.path().join(".git/stacked.toml")).unwrap();
    // feat/b gone, feat/a and feat/c still present
    assert!(
        !toml.contains("\"feat/b\"") && !toml.contains("feat/b\n"),
        "feat/b should be absent, got:\n{}",
        toml
    );
    assert!(toml.contains("feat/a"));
    assert!(toml.contains("feat/c"));
}

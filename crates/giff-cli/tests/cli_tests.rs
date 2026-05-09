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
    let resume = r#"{"stack_id":"s1","pending_frame_ids":["f2","f3"],"original_branch":"feat/a"}"#;
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

// --- Tree-shape support -----------------------------------------------------

/// Build a Y-shaped stack:
///   main → feat/root → feat/left
///                    ↘ feat/right
fn build_y_stack(repo_path: &std::path::Path) {
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo_path)
        .args(["new", "feat/root"])
        .assert()
        .success();
    git_commit_file(repo_path, "root.txt", "root work");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo_path)
        .args(["new", "feat/left"])
        .assert()
        .success();
    git_commit_file(repo_path, "left.txt", "left work");
    // Hop back to root and create a sibling.
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo_path)
        .args(["checkout", "feat/root"])
        .assert()
        .success();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo_path)
        .args(["new", "feat/right"])
        .assert()
        .success();
    git_commit_file(repo_path, "right.txt", "right work");
}

#[test]
fn tree_giff_new_creates_sibling_with_correct_parent() {
    let repo = init_git_repo();
    build_y_stack(repo.path());

    let toml = std::fs::read_to_string(repo.path().join(".git/stacked.toml")).unwrap();
    // Both children should reference feat/root's frame as parent. We can't easily
    // know the UUID without parsing, so check that there are exactly two `parent =`
    // lines (one per non-root frame) and that feat/root has none.
    let parent_lines = toml.matches("parent =").count();
    assert_eq!(parent_lines, 2, "expected 2 parent= lines, toml:\n{}", toml);
    assert!(toml.contains("feat/root"));
    assert!(toml.contains("feat/left"));
    assert!(toml.contains("feat/right"));
}

#[test]
fn tree_giff_log_renders_branch_for_y_shape() {
    let repo = init_git_repo();
    build_y_stack(repo.path());
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("log")
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {:?}", out.stderr);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Both leaves rendered, and at least one branch connector visible.
    assert!(stdout.contains("feat/root"), "{}", stdout);
    assert!(stdout.contains("feat/left"), "{}", stdout);
    assert!(stdout.contains("feat/right"), "{}", stdout);
    assert!(
        stdout.contains("├─") || stdout.contains("└─"),
        "expected tree connector in log output, got:\n{}",
        stdout
    );
}

#[test]
fn tree_giff_next_errors_when_ambiguous() {
    let repo = init_git_repo();
    build_y_stack(repo.path());
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["checkout", "feat/root"])
        .assert()
        .success();
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("next")
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("ambiguous")
            && stderr.contains("feat/left")
            && stderr.contains("feat/right"),
        "expected ambiguity message, got:\n{}",
        stderr
    );
}

#[test]
fn tree_giff_drop_root_promotes_children_to_roots() {
    let repo = init_git_repo();
    build_y_stack(repo.path());
    let drop_out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["stack", "drop", "feat/root"])
        .output()
        .unwrap();
    let toml = std::fs::read_to_string(repo.path().join(".git/stacked.toml")).unwrap();
    assert!(
        drop_out.status.success(),
        "drop failed: stderr={:?}\ntoml=\n{}",
        String::from_utf8_lossy(&drop_out.stderr),
        toml
    );
    // After dropping the only root, both children should become parentless.
    assert!(
        !toml.contains("branch = \"feat/root\""),
        "feat/root should be gone, toml=\n{}",
        toml
    );
    assert_eq!(
        toml.matches("parent =").count(),
        0,
        "no remaining parent= lines expected, got:\n{}",
        toml
    );
}

#[test]
fn tree_giff_checkout_by_position_rejects_tree() {
    let repo = init_git_repo();
    build_y_stack(repo.path());
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["checkout", "2"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("tree") && stderr.contains("linear"),
        "expected linear-only message, got:\n{}",
        stderr
    );
}

#[test]
fn tree_giff_status_shows_tree_shape_and_path() {
    let repo = init_git_repo();
    build_y_stack(repo.path());
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("status")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("tree"),
        "expected 'tree' in status:\n{}",
        stdout
    );
    // We are on feat/right (last `giff new`), so path should include both root and right.
    assert!(stdout.contains("feat/root"), "{}", stdout);
    assert!(stdout.contains("feat/right"), "{}", stdout);
}

// --- One-commit-per-frame enforcement --------------------------------------

/// Stage a new file in `dir` without committing.
fn stage_file(dir: &std::path::Path, name: &str, contents: &str) {
    std::fs::write(dir.join(name), contents).unwrap();
    StdCommand::new("git")
        .args(["add", "."])
        .current_dir(dir)
        .output()
        .unwrap();
}

#[test]
fn giff_commit_first_commit_succeeds() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    stage_file(repo.path(), "a.txt", "first");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "first commit"])
        .assert()
        .success();
}

#[test]
fn giff_commit_second_commit_blocked() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    stage_file(repo.path(), "a.txt", "first");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "first"])
        .assert()
        .success();

    stage_file(repo.path(), "b.txt", "second");
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "second"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("one commit per frame")
            && stderr.contains("giff new")
            && stderr.contains("--amend"),
        "expected enforcement message, got:\n{}",
        stderr
    );
}

#[test]
fn giff_commit_blocks_when_a_prior_git_commit_already_exists() {
    // Even if the first commit was made via plain `git commit`, `giff commit` must still
    // refuse a second commit — the rule is about state, not API.
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    git_commit_file(repo.path(), "a.txt", "first via git");

    stage_file(repo.path(), "b.txt", "second via giff");
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "second"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("one commit per frame"));
}

#[test]
fn giff_commit_amend_always_allowed() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    stage_file(repo.path(), "a.txt", "first");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "first"])
        .assert()
        .success();

    // Amend with new content keeps the one-commit invariant.
    stage_file(repo.path(), "a.txt", "first updated");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "--amend", "-m", "first (revised)"])
        .assert()
        .success();
}

#[test]
fn giff_commit_amend_no_message_keeps_existing() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    stage_file(repo.path(), "a.txt", "first");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "original message"])
        .assert()
        .success();

    stage_file(repo.path(), "a.txt", "first revised");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "--amend"])
        .assert()
        .success();

    // Most-recent commit message should still be the original.
    let out = StdCommand::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let msg = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(msg, "original message");
}

#[test]
fn giff_commit_outside_a_stack_is_rejected() {
    let repo = init_git_repo();
    stage_file(repo.path(), "x.txt", "x");
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "x"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not in a stack"));
}

/// Run a plain `git` command with the giff binary's directory prepended to PATH, so the
/// installed pre-commit hook can shell out to `giff parent-branch`.
fn git_with_giff_on_path(dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    let bin = assert_cmd::cargo::cargo_bin("giff");
    let bin_dir = bin.parent().unwrap();
    let existing = std::env::var("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin_dir.display(), existing);
    StdCommand::new("git")
        .args(args)
        .current_dir(dir)
        .env("PATH", new_path)
        .output()
        .unwrap()
}

#[test]
fn pre_commit_hook_installed_after_giff_new() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    let hook = repo.path().join(".git/hooks/pre-commit");
    assert!(hook.exists(), "pre-commit hook should be installed");
    let body = std::fs::read_to_string(&hook).unwrap();
    assert!(body.contains("giff:one-commit-rule v1"));
}

#[test]
fn direct_git_commit_blocked_for_second_commit_on_frame() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();

    // First commit via plain git — hook allows it (count is 0 → 1).
    stage_file(repo.path(), "a.txt", "first");
    let out = git_with_giff_on_path(repo.path(), &["commit", "-m", "first"]);
    assert!(
        out.status.success(),
        "first commit should succeed: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Second commit via plain git — hook MUST block.
    stage_file(repo.path(), "b.txt", "second");
    let out = git_with_giff_on_path(repo.path(), &["commit", "-m", "second"]);
    assert!(
        !out.status.success(),
        "second direct git commit should be blocked"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("one commit per frame"),
        "expected enforcement message in hook stderr, got:\n{}",
        stderr
    );
}

#[test]
fn direct_git_commit_amend_passes_hook() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();

    stage_file(repo.path(), "a.txt", "first");
    let out = git_with_giff_on_path(repo.path(), &["commit", "-m", "first"]);
    assert!(out.status.success());

    stage_file(repo.path(), "a.txt", "first revised");
    let out = git_with_giff_on_path(repo.path(), &["commit", "--amend", "-m", "first revised"]);
    assert!(
        out.status.success(),
        "amend should bypass the hook, stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn parent_branch_subcommand_prints_trunk_for_root_frame() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("parent-branch")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(stdout, "main");
}

#[test]
fn parent_branch_subcommand_prints_parent_branch_for_child_frame() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    git_commit_file(repo.path(), "a.txt", "first");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .arg("parent-branch")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(stdout, "feat/a");
}

#[test]
fn squash_amends_parent_keeping_one_commit_rule() {
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    git_commit_file(repo.path(), "a.txt", "frame a");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    git_commit_file(repo.path(), "b.txt", "frame b");

    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["stack", "squash", "feat/b"])
        .assert()
        .success();

    // After squash, feat/a should still have exactly one commit ahead of main.
    let out = StdCommand::new("git")
        .args(["rev-list", "--count", "main..feat/a"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let count: usize = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap();
    assert_eq!(
        count, 1,
        "expected 1 commit on feat/a after squash, got {}",
        count
    );
}

#[test]
fn squash_refuses_when_parent_has_no_commits() {
    let repo = init_git_repo();
    // Create feat/a with no commits, then feat/b on top.
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
    git_commit_file(repo.path(), "b.txt", "frame b");

    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["stack", "squash", "feat/b"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no commits to absorb"),
        "expected no-parent-commits message, got:\n{}",
        stderr
    );
}

#[test]
fn giff_commit_after_new_frame_is_unblocked() {
    // Once you start a new frame on top, the next `giff commit` should work again.
    let repo = init_git_repo();
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/a"])
        .assert()
        .success();
    stage_file(repo.path(), "a.txt", "first");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "first"])
        .assert()
        .success();

    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["new", "feat/b"])
        .assert()
        .success();
    stage_file(repo.path(), "b.txt", "second");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["commit", "-m", "second"])
        .assert()
        .success();
}

// --- giff publish (frame creation + commit in one step) -------------------

#[test]
fn publish_creates_frame_and_commits_in_one_step() {
    let repo = init_git_repo();
    stage_file(repo.path(), "a.txt", "first work");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "Add token signing"])
        .assert()
        .success();

    // Branch derived from the message.
    assert_eq!(current_branch(repo.path()), "add-token-signing");

    // Exactly one commit on the new branch beyond main.
    let out = StdCommand::new("git")
        .args(["rev-list", "--count", "main..HEAD"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let count: usize = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap();
    assert_eq!(count, 1, "expected 1 commit, got {}", count);

    // Commit message preserved verbatim.
    let out = StdCommand::new("git")
        .args(["log", "-1", "--pretty=%s"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&out.stdout).trim(),
        "Add token signing"
    );
}

#[test]
fn publish_handles_conventional_prefix() {
    let repo = init_git_repo();
    stage_file(repo.path(), "a.txt", "fix work");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "feat: token signing"])
        .assert()
        .success();
    assert_eq!(current_branch(repo.path()), "feat/token-signing");
}

#[test]
fn publish_branch_override_wins() {
    let repo = init_git_repo();
    stage_file(repo.path(), "a.txt", "x");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "Anything", "-b", "feat/custom-name"])
        .assert()
        .success();
    assert_eq!(current_branch(repo.path()), "feat/custom-name");
}

#[test]
fn publish_fails_fast_when_nothing_staged() {
    let repo = init_git_repo();
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "Should not happen"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("nothing staged"),
        "expected 'nothing staged' message, got:\n{}",
        stderr
    );
    // Crucially, the branch must NOT have been created.
    assert_ne!(current_branch(repo.path()), "should-not-happen");
}

#[test]
fn publish_with_a_flag_auto_stages_tracked_changes() {
    let repo = init_git_repo();
    // Commit a tracked file first so it can be modified later without staging.
    git_commit_file(repo.path(), "tracked.txt", "initial");
    // Modify without staging.
    std::fs::write(repo.path().join("tracked.txt"), "modified").unwrap();

    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "-a", "Tweak file"])
        .assert()
        .success();
    assert_eq!(current_branch(repo.path()), "tweak-file");
    let out = StdCommand::new("git")
        .args(["rev-list", "--count", "main..HEAD"])
        .current_dir(repo.path())
        .output()
        .unwrap();
    let count: usize = String::from_utf8_lossy(&out.stdout).trim().parse().unwrap();
    assert_eq!(count, 1);
}

#[test]
fn publish_rejects_message_that_slugifies_empty() {
    let repo = init_git_repo();
    stage_file(repo.path(), "a.txt", "x");
    let out = Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "!!!"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("could not derive"));
}

#[test]
fn publish_chains_to_build_a_stack() {
    // Two publishes in a row should build a two-frame linear stack.
    let repo = init_git_repo();
    stage_file(repo.path(), "a.txt", "a");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "feat: layer one"])
        .assert()
        .success();

    stage_file(repo.path(), "b.txt", "b");
    Command::cargo_bin("giff")
        .unwrap()
        .current_dir(repo.path())
        .args(["publish", "feat: layer two"])
        .assert()
        .success();

    assert_eq!(current_branch(repo.path()), "feat/layer-two");
    let toml = std::fs::read_to_string(repo.path().join(".git/stacked.toml")).unwrap();
    assert!(toml.contains("feat/layer-one"));
    assert!(toml.contains("feat/layer-two"));
}

// Pre-commit hook installer.
//
// Installs `.git/hooks/pre-commit` that enforces "one commit per frame" against direct
// `git commit` calls (not just `giff commit`). Idempotent: safely re-runs every time a giff
// command touches a stack-managed repo. Will not clobber a user's existing hook.

use anyhow::Result;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

const MARKER: &str = "# giff:one-commit-rule v1";

const HOOK_SCRIPT: &str = r#"#!/usr/bin/env bash
# giff:one-commit-rule v1
# Enforces "one commit per frame" for branches managed by giff. Pass-through for everything
# else, so non-giff repos and non-giff branches behave normally.

set -e

git_dir=$(git rev-parse --git-dir 2>/dev/null) || exit 0

# Pass through during rebase / cherry-pick / merge — those preserve the existing commit count
# rather than adding a new one. Without this guard, `giff sync` (which rebases) would deadlock.
[ -d "$git_dir/rebase-merge" ] && exit 0
[ -d "$git_dir/rebase-apply" ] && exit 0
[ -f "$git_dir/CHERRY_PICK_HEAD" ] && exit 0
[ -f "$git_dir/MERGE_HEAD" ] && exit 0

# Pass through if amending — replaces a commit, doesn't add one.
if ps -o args= -p $PPID 2>/dev/null | grep -q -- "--amend"; then
  exit 0
fi

# Pass through if this repo isn't giff-managed.
[ -f "$git_dir/stacked.toml" ] || exit 0

# Pass through if `giff` isn't on PATH (don't lock the user out — warn instead).
if ! command -v giff >/dev/null 2>&1; then
  echo "giff: pre-commit hook installed but \`giff\` not on PATH; skipping check" >&2
  exit 0
fi

branch=$(git rev-parse --abbrev-ref HEAD 2>/dev/null) || exit 0

# Ask giff for the current frame's parent branch. Empty output means this branch isn't tracked.
parent=$(giff parent-branch 2>/dev/null || true)
[ -z "$parent" ] && exit 0

count=$(git rev-list --count "$parent..HEAD" 2>/dev/null || echo 0)
if [ "$count" -ge 1 ]; then
  echo "" >&2
  echo "giff: frame '$branch' already has $count commit(s) ahead of '$parent'." >&2
  echo "giff: one commit per frame is enforced." >&2
  echo "giff:   → start a new frame: giff new <branch-name>" >&2
  echo "giff:   → amend instead:     giff commit --amend   (or git commit --amend)" >&2
  exit 1
fi
exit 0
"#;

#[derive(Debug, PartialEq, Eq)]
pub enum HookInstallStatus {
    /// Wrote a fresh hook (none was present).
    Installed,
    /// Replaced an older giff-owned hook with the current version.
    Updated,
    /// Already installed at the current version; nothing to do.
    AlreadyCurrent,
    /// A non-giff hook is in place; we left it alone.
    UserHookPresent,
}

/// Install the pre-commit hook in `git_dir/hooks/`. `git_dir` is the `.git/` directory
/// (or worktree's git dir), not the repo root.
pub fn install_pre_commit_hook(git_dir: &Path) -> Result<HookInstallStatus> {
    let hooks_dir = git_dir.join("hooks");
    fs::create_dir_all(&hooks_dir)?;
    let hook_path = hooks_dir.join("pre-commit");

    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path)?;
        if !existing.contains(MARKER) {
            return Ok(HookInstallStatus::UserHookPresent);
        }
        if existing == HOOK_SCRIPT {
            return Ok(HookInstallStatus::AlreadyCurrent);
        }
        fs::write(&hook_path, HOOK_SCRIPT)?;
        chmod_executable(&hook_path)?;
        return Ok(HookInstallStatus::Updated);
    }

    fs::write(&hook_path, HOOK_SCRIPT)?;
    chmod_executable(&hook_path)?;
    Ok(HookInstallStatus::Installed)
}

fn chmod_executable(path: &Path) -> Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms)?;
    Ok(())
}

/// Best-effort installer used by every command that touches a stack store. Logs a warning if
/// a user-owned hook is present so they know enforcement is opt-in, but never fails the caller.
pub fn ensure_installed_quiet(git_dir: &Path) {
    match install_pre_commit_hook(git_dir) {
        Ok(HookInstallStatus::Installed) => {
            eprintln!(
                "giff: installed pre-commit hook at {}",
                git_dir.join("hooks/pre-commit").display()
            );
        }
        Ok(HookInstallStatus::Updated) => {
            eprintln!("giff: updated pre-commit hook to current version");
        }
        Ok(HookInstallStatus::UserHookPresent) => {
            eprintln!(
                "giff: warning: existing pre-commit hook detected at {}",
                git_dir.join("hooks/pre-commit").display()
            );
            eprintln!(
                "giff:   → one-commit-per-frame is NOT enforced for direct `git commit` in this repo."
            );
            eprintln!(
                "giff:   → To enable: append the giff hook script to your existing pre-commit, or remove it and re-run any giff command."
            );
        }
        Ok(HookInstallStatus::AlreadyCurrent) => {}
        Err(e) => {
            eprintln!("giff: warning: could not install pre-commit hook: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn fresh_install_writes_executable_hook() {
        let td = TempDir::new().unwrap();
        let status = install_pre_commit_hook(td.path()).unwrap();
        assert_eq!(status, HookInstallStatus::Installed);
        let hook = td.path().join("hooks/pre-commit");
        assert!(hook.exists());
        let perms = fs::metadata(&hook).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o755);
        let body = fs::read_to_string(&hook).unwrap();
        assert!(body.contains(MARKER));
    }

    #[test]
    fn second_install_is_idempotent() {
        let td = TempDir::new().unwrap();
        install_pre_commit_hook(td.path()).unwrap();
        let status = install_pre_commit_hook(td.path()).unwrap();
        assert_eq!(status, HookInstallStatus::AlreadyCurrent);
    }

    #[test]
    fn install_does_not_overwrite_user_hook() {
        let td = TempDir::new().unwrap();
        fs::create_dir_all(td.path().join("hooks")).unwrap();
        let user_hook = "#!/bin/sh\necho hi\n";
        fs::write(td.path().join("hooks/pre-commit"), user_hook).unwrap();

        let status = install_pre_commit_hook(td.path()).unwrap();
        assert_eq!(status, HookInstallStatus::UserHookPresent);
        let body = fs::read_to_string(td.path().join("hooks/pre-commit")).unwrap();
        assert_eq!(body, user_hook);
    }

    #[test]
    fn install_replaces_old_giff_hook() {
        let td = TempDir::new().unwrap();
        fs::create_dir_all(td.path().join("hooks")).unwrap();
        let old_giff_hook = format!("#!/bin/sh\n{}\n# old version\nexit 0\n", MARKER);
        fs::write(td.path().join("hooks/pre-commit"), &old_giff_hook).unwrap();

        let status = install_pre_commit_hook(td.path()).unwrap();
        assert_eq!(status, HookInstallStatus::Updated);
        let body = fs::read_to_string(td.path().join("hooks/pre-commit")).unwrap();
        assert_eq!(body, HOOK_SCRIPT);
    }
}

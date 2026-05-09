// crates/giff-cli/src/commands/publish.rs
//
// Combines `giff new` + `giff commit` into one action. The message is the source of truth:
// it becomes the commit message and (slugified) the branch name. The user types one thing.
//
// Pre-checks (fail BEFORE creating any branch):
//   - There must be something to commit (staged, or unstaged tracked changes when -a is set).
//   - The slugified message must yield a non-empty git-ref-safe branch name (or -b set).
//
// We compose `new::run` + `commit::run`. They each load and write the store (which is a tiny
// extra cost, but keeps the responsibility split clear and means each command stays the
// single source of truth for its own validation + side-effects).

use crate::commands::{commit, new};
use anyhow::Result;
use giff_git::ShellGitBackend;

pub fn run(message: &str, branch_override: Option<&str>, all: bool) -> Result<()> {
    if message.trim().is_empty() {
        anyhow::bail!("message must not be empty");
    }

    let backend = ShellGitBackend::new(std::env::current_dir()?);

    // Resolve the branch name first so we can fail fast on a bad slug before touching git.
    let branch = match branch_override {
        Some(b) if !b.trim().is_empty() => b.trim().to_string(),
        _ => slugify(message).ok_or_else(|| {
            anyhow::anyhow!(
                "could not derive a branch name from `{}` — pass -b/--branch to set explicitly",
                message
            )
        })?,
    };

    // Make sure there's actually work to commit. `git diff --quiet` exits 0 when clean.
    let staged_clean = backend.git_raw(&["diff", "--cached", "--quiet"]).is_ok();
    let unstaged_clean = backend.git_raw(&["diff", "--quiet"]).is_ok();

    if all {
        if staged_clean && unstaged_clean {
            anyhow::bail!("nothing to commit (working tree clean)");
        }
    } else if staged_clean {
        anyhow::bail!(
            "nothing staged. stage with `git add ...`, or pass `-a` to auto-stage tracked changes"
        );
    }

    new::run(&branch)?;
    commit::run(Some(message), false, all)?;
    Ok(())
}

/// Convert a message like "Add token signing" into a branch-friendly slug.
/// Conventional-commit prefixes like "feat:" / "fix:" become path segments: "feat/...".
pub fn slugify(message: &str) -> Option<String> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Conventional commit prefix: a short alnum word, ":" separator, then body.
    if let Some(colon) = trimmed.find(':') {
        let prefix = trimmed[..colon].trim();
        let rest = trimmed[colon + 1..].trim();
        if !prefix.is_empty()
            && !rest.is_empty()
            && prefix.len() <= 16
            && prefix
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            let body = slug_body(rest)?;
            return Some(format!("{}/{}", prefix.to_lowercase(), body));
        }
    }

    slug_body(trimmed)
}

const MAX_SLUG_LEN: usize = 60;

fn slug_body(s: &str) -> Option<String> {
    let mut out = String::new();
    let mut last_hyphen = false;
    for c in s.chars() {
        let lower = c.to_ascii_lowercase();
        // Keep ASCII alnum, underscore, dot. Drop anything else, replacing with a hyphen.
        // (Slashes are not kept in the body — only the conventional-prefix path uses `/`.)
        if lower.is_ascii_alphanumeric() || lower == '_' || lower == '.' {
            out.push(lower);
            last_hyphen = false;
        } else if !out.is_empty() && !last_hyphen {
            out.push('-');
            last_hyphen = true;
        }
    }
    let trimmed: String = out.trim_matches('-').chars().take(MAX_SLUG_LEN).collect();
    let trimmed: String = trimmed.trim_matches('-').to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugifies_basic_message() {
        assert_eq!(slugify("Add token signing"), Some("add-token-signing".into()));
    }

    #[test]
    fn slugifies_conventional_prefix() {
        assert_eq!(
            slugify("feat: token signing"),
            Some("feat/token-signing".into())
        );
        assert_eq!(slugify("fix: redirect bug"), Some("fix/redirect-bug".into()));
        assert_eq!(slugify("chore: bump deps"), Some("chore/bump-deps".into()));
    }

    #[test]
    fn drops_punctuation_and_collapses_hyphens() {
        assert_eq!(
            slugify("Refactor (round 2)!!"),
            Some("refactor-round-2".into())
        );
    }

    #[test]
    fn ignores_non_alnum_only_garbage() {
        assert_eq!(slugify(""), None);
        assert_eq!(slugify("   "), None);
        assert_eq!(slugify("!!!"), None);
        assert_eq!(slugify("---"), None);
    }

    #[test]
    fn caps_length_at_60() {
        let long = "the-quick-brown-fox-jumps-over-the-lazy-dog-and-then-keeps-going";
        let s = slugify(long).unwrap();
        assert!(s.len() <= MAX_SLUG_LEN, "got {} chars: {}", s.len(), s);
    }

    #[test]
    fn does_not_treat_long_word_as_conventional_prefix() {
        // Prefix must be short alnum; `something-very-long` shouldn't trigger the path mode.
        let long_prefix = "this-is-a-very-long-word-that-should-not-be-treated-as-prefix: rest";
        let s = slugify(long_prefix).unwrap();
        assert!(!s.contains('/'), "expected no slash, got: {}", s);
    }

    #[test]
    fn preserves_existing_dots_and_underscores() {
        assert_eq!(slugify("v1.2 release"), Some("v1.2-release".into()));
        assert_eq!(slugify("API_v2 stuff"), Some("api_v2-stuff".into()));
    }
}

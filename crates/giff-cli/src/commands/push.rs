// crates/giff-cli/src/commands/push.rs
use crate::config::{find_stack_store_path, read_stack_store, write_stack_store, GlobalConfig};
use anyhow::{Context, Result};
use giff_core::RemoteStackMeta;
use giff_git::{GitBackend, ShellGitBackend};
use giff_github::{CreatePrParams, ForgeBackend, GitHubForge, UpdatePrParams};

pub fn run() -> Result<()> {
    let cfg = GlobalConfig::load()?;
    let token = std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| {
            if cfg.github.token.is_empty() {
                None
            } else {
                Some(cfg.github.token.clone())
            }
        })
        .context(
            "no GitHub token found — set GITHUB_TOKEN or add token to ~/.config/giff/config.toml",
        )?;

    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;

    let stack_id = {
        let (stack, _) = store.find_stack_for_branch(&current).ok_or_else(|| {
            anyhow::anyhow!(
                "branch `{}` is not in a stack — run `giff new` first",
                current
            )
        })?;
        stack.id.clone()
    };

    // Detect repo from git remote
    let remote_url = backend
        .git_raw(&["remote", "get-url", "origin"])
        .context("no git remote named 'origin'")?;
    let repo =
        parse_github_repo(&remote_url).context("could not parse GitHub repo from remote URL")?;

    let forge = GitHubForge::new(token, repo, cfg.github.base_url.clone());

    let stack = store
        .stacks
        .iter()
        .find(|s| s.id == stack_id)
        .unwrap()
        .clone();
    let total = stack.frames.len();

    for (i, frame) in stack.ordered_frames().iter().enumerate() {
        let position = i + 1;
        let base = if i == 0 {
            stack.trunk.clone()
        } else {
            stack.ordered_frames()[i - 1].branch.clone()
        };

        let meta = RemoteStackMeta {
            stack_id: stack.id.clone(),
            frame_id: frame.id.clone(),
            position,
            total,
        };
        let body = format!(
            "Part {}/{} of stack `{}`.\n\n{}",
            position,
            total,
            stack.name,
            meta.to_pr_block()
        );

        backend.push(&frame.branch, true)?;

        let frame_id = frame.id.clone();
        let frame_branch = frame.branch.clone();
        let frame_pr_number = frame.pr_number;

        let pr_number = if let Some(existing) = frame_pr_number {
            forge.update_pr(
                existing,
                UpdatePrParams {
                    body: Some(body),
                    base: Some(base),
                },
            )?;
            existing
        } else {
            let pr = forge.create_pr(CreatePrParams {
                title: frame_branch.clone(),
                body,
                head: frame_branch.clone(),
                base,
                draft: cfg.defaults.draft_prs,
            })?;
            pr.number
        };

        // Update pr_number in store
        let s = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
        let f = s.frames.iter_mut().find(|f| f.id == frame_id).unwrap();
        f.pr_number = Some(pr_number);

        println!("  {} → PR #{}", frame_branch, pr_number);
    }

    write_stack_store(&store_path, &store)?;
    Ok(())
}

pub fn parse_github_repo(remote_url: &str) -> Option<String> {
    // Handles: git@github.com:owner/repo.git and https://github.com/owner/repo.git
    let url = remote_url.trim().trim_end_matches(".git");
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        return Some(rest.to_string());
    }
    if let Some(rest) = url.strip_prefix("https://github.com/") {
        return Some(rest.to_string());
    }
    None
}

use crate::commands::push::parse_github_repo;
use crate::config::{find_stack_store_path, read_stack_store, GlobalConfig};
use anyhow::Result;
use giff_git::{GitBackend, ShellGitBackend};
use giff_github::{ForgeBackend, GitHubForge};

pub fn run() -> Result<()> {
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;

    if store.stacks.is_empty() {
        println!("No stacks. Run `giff new <branch>` to create one.");
        return Ok(());
    }

    // Try to set up GitHub forge for live PR status — optional, falls back gracefully.
    let forge = build_forge(&backend);

    for stack in &store.stacks {
        println!("stack: {} (trunk: {})", stack.name, stack.trunk);
        println!("● {}", stack.trunk);
        for frame in stack.ordered_frames() {
            let marker = if frame.branch == current {
                " ← you are here"
            } else {
                ""
            };
            let pr_label = match frame.pr_number {
                None => "no PR".to_string(),
                Some(n) => {
                    let state = forge
                        .as_ref()
                        .and_then(|f| f.get_pr(n).ok())
                        .map(|pr| pr.state.clone())
                        .unwrap_or_else(|| format!("#{}", n));
                    // state from GitHub is "open", "closed", or "merged" (via merged_at)
                    format!("PR #{} [{}]", n, state)
                }
            };
            println!("│");
            println!("◉ {}  [{}]{}", frame.branch, pr_label, marker);
        }
        println!();
    }
    Ok(())
}

fn build_forge(backend: &ShellGitBackend) -> Option<GitHubForge> {
    let cfg = GlobalConfig::load().ok()?;
    let token = std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| {
            if cfg.github.token.is_empty() {
                None
            } else {
                Some(cfg.github.token.clone())
            }
        })?;
    let remote_url = backend.git_raw(&["remote", "get-url", "origin"]).ok()?;
    let repo = parse_github_repo(&remote_url)?;
    Some(GitHubForge::new(token, repo, cfg.github.base_url))
}

// crates/giff-cli/src/commands/stack/land.rs
use crate::commands::push::parse_github_repo;
use crate::config::{find_stack_store_path, read_stack_store, write_stack_store, GlobalConfig};
use anyhow::Result;
use giff_git::{GitBackend, ShellGitBackend};
use giff_github::{ForgeBackend, GitHubForge, UpdatePrParams};

pub fn run(merge_method: &str) -> Result<()> {
    match merge_method {
        "merge" | "squash" | "rebase" => {}
        other => anyhow::bail!(
            "unknown merge method `{}` — choose merge, squash, or rebase",
            other
        ),
    }

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
        .ok_or_else(|| anyhow::anyhow!("no GitHub token — set GITHUB_TOKEN"))?;

    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;

    let stack_id = store
        .find_stack_for_branch(&current)
        .map(|(s, _)| s.id.clone())
        .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;

    let remote_url = backend.git_raw(&["remote", "get-url", "origin"])?;
    let repo = parse_github_repo(&remote_url)
        .ok_or_else(|| anyhow::anyhow!("could not parse GitHub repo from remote"))?;
    let forge = GitHubForge::new(token, repo, cfg.github.base_url);

    let stack = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
    if stack.frames.is_empty() {
        anyhow::bail!("stack is empty");
    }

    let bottom = stack.frames.remove(0);
    let pr_number = bottom
        .pr_number
        .ok_or_else(|| anyhow::anyhow!("bottom frame has no PR — run `giff push` first"))?;

    // Verify the PR is open before merging
    let pr = forge.get_pr(pr_number)?;
    if pr.state != "open" {
        anyhow::bail!(
            "PR #{} is already {} — nothing to land",
            pr_number,
            pr.state
        );
    }

    // Merge the PR on GitHub
    forge.merge_pr(pr_number, merge_method)?;
    println!(
        "Merged PR #{} ({}) via {}.",
        pr_number, bottom.branch, merge_method
    );

    // Re-target new bottom frame to trunk
    if let Some(new_bottom) = stack.frames.first_mut() {
        new_bottom.parent = None;
        if let Some(pr) = new_bottom.pr_number {
            let trunk = stack.trunk.clone();
            forge.update_pr(
                pr,
                UpdatePrParams {
                    body: None,
                    base: Some(trunk),
                },
            )?;
            println!("Retargeted PR #{} to {}.", pr, stack.trunk);
        }
    }

    let trunk = stack.trunk.clone();
    write_stack_store(&store_path, &store)?;
    println!(
        "Run `giff sync` to rebase remaining frames onto the updated {}.",
        trunk
    );
    Ok(())
}

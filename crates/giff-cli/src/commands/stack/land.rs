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
    stack.validate()?;
    if stack.frames.is_empty() {
        anyhow::bail!("stack is empty");
    }

    // `giff stack land` only makes sense when there is exactly one root — otherwise
    // "the bottom" is ambiguous. Multi-root stacks must land each root in its own command,
    // which currently means restructuring the stack first.
    let root_ids: Vec<_> = stack.roots().iter().map(|f| f.id.clone()).collect();
    if root_ids.len() != 1 {
        anyhow::bail!(
            "stack has {} roots — `giff stack land` requires exactly one root frame",
            root_ids.len()
        );
    }
    let root_id = root_ids.into_iter().next().unwrap();
    let root_idx = stack.frames.iter().position(|f| f.id == root_id).unwrap();
    let bottom = stack.frames.remove(root_idx);

    let pr_number = bottom
        .pr_number
        .ok_or_else(|| anyhow::anyhow!("root frame has no PR — run `giff push` first"))?;

    let pr = forge.get_pr(pr_number)?;
    if pr.state != "open" {
        anyhow::bail!(
            "PR #{} is already {} — nothing to land",
            pr_number,
            pr.state
        );
    }

    forge.merge_pr(pr_number, merge_method)?;
    println!(
        "Merged PR #{} ({}) via {}.",
        pr_number, bottom.branch, merge_method
    );

    // Every direct child of the landed frame becomes a new root: parent = None,
    // and any open PR retargets to the trunk.
    let trunk = stack.trunk.clone();
    let new_root_ids: Vec<_> = stack
        .frames
        .iter()
        .filter(|f| f.parent.as_ref() == Some(&bottom.id))
        .map(|f| f.id.clone())
        .collect();

    // The merge has already happened. From here on, partial failure is much worse than a
    // complete one — we'd leave GitHub with some children retargeted and some still pointing
    // at the (now-deleted) merged branch. Warn-and-continue, then summarise so the user knows
    // exactly what to retry.
    let mut retarget_failures: Vec<u64> = Vec::new();
    for id in &new_root_ids {
        let f = stack.frames.iter_mut().find(|f| &f.id == id).unwrap();
        f.parent = None;
        if let Some(pr_num) = f.pr_number {
            match forge.update_pr(
                pr_num,
                UpdatePrParams {
                    body: None,
                    base: Some(trunk.clone()),
                },
            ) {
                Ok(_) => println!("Retargeted PR #{} to {}.", pr_num, trunk),
                Err(e) => {
                    eprintln!("warning: failed to retarget PR #{}: {}", pr_num, e);
                    retarget_failures.push(pr_num);
                }
            }
        }
    }
    if !retarget_failures.is_empty() {
        eprintln!();
        eprintln!(
            "warning: {} PR(s) still point at the merged branch on GitHub: {:?}",
            retarget_failures.len(),
            retarget_failures
        );
        eprintln!("  → Re-run `giff sync` to retry retargeting them.");
    }

    stack.validate()?;
    write_stack_store(&store_path, &store)?;
    println!(
        "Run `giff sync` to rebase remaining frames onto the updated {}.",
        trunk
    );
    Ok(())
}

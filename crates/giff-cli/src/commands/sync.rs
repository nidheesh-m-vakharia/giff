// crates/giff-cli/src/commands/sync.rs
use crate::commands::push::parse_github_repo;
use crate::config::{find_stack_store_path, read_stack_store, write_stack_store, GlobalConfig};
use anyhow::{bail, Result};
use giff_core::{FrameId, Stack, StackFrame, StackId, StackStore};
use giff_git::{GitBackend, RebaseOutcome, ShellGitBackend};
use giff_github::{ForgeBackend, GitHubForge, UpdatePrParams};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Resume state stored at .git/giff_sync_resume.json on conflict.
///
/// `pending_frame_ids` is the list of frames *still to rebase* in topological order, NOT
/// including the one that conflicted (that one is being completed by the user via
/// `git rebase --continue`). This makes resume robust to mid-conflict edits to the stack:
/// each pending frame's parent is looked up fresh on resume.
#[derive(Serialize, Deserialize)]
struct SyncResume {
    stack_id: String,
    pending_frame_ids: Vec<String>,
    original_branch: String,
}

fn resume_state_path() -> Result<PathBuf> {
    let store_path = find_stack_store_path()?;
    let git_dir = store_path.parent().unwrap().to_path_buf();
    Ok(git_dir.join("giff_sync_resume.json"))
}

fn save_resume(state: &SyncResume) -> Result<()> {
    let path = resume_state_path()?;
    let json = serde_json::to_string(state)?;
    std::fs::write(path, json)?;
    Ok(())
}

fn load_resume() -> Result<Option<SyncResume>> {
    let path = resume_state_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let json = std::fs::read_to_string(&path)?;
    Ok(Some(serde_json::from_str(&json)?))
}

fn clear_resume() -> Result<()> {
    let path = resume_state_path()?;
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn run(resume: bool) -> Result<()> {
    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;
    let backend = ShellGitBackend::new(std::env::current_dir()?);

    if resume {
        let state = load_resume()?.ok_or_else(|| {
            anyhow::anyhow!("no interrupted sync found — run `giff sync` to start")
        })?;

        if backend.is_rebase_in_progress() {
            bail!(
                "a rebase is still in progress.\n\
                Resolve the conflicts, stage your changes, then run:\n\
                  git rebase --continue\n\
                and then retry: giff sync --continue"
            );
        }

        let stack = store
            .stacks
            .iter()
            .find(|s| s.id.0 == state.stack_id)
            .ok_or_else(|| anyhow::anyhow!("stack from resume state not found"))?;
        stack.validate()?;

        // Translate the saved frame ids back into frames; skip ones the user removed since.
        let pending: Vec<StackFrame> = state
            .pending_frame_ids
            .iter()
            .filter_map(|id| stack.frame(&FrameId(id.clone())).cloned())
            .collect();

        if pending.is_empty() {
            println!("Nothing left to restack.");
        } else {
            println!(
                "Resuming sync — {} frame{} remaining.",
                pending.len(),
                if pending.len() == 1 { "" } else { "s" }
            );
            restack(
                &backend,
                stack,
                &pending,
                &state.stack_id,
                &state.original_branch,
            )?;
        }
        clear_resume()?;
        backend.checkout(&state.original_branch)?;
    } else {
        if let Some(state) = load_resume()? {
            bail!(
                "a previous sync was interrupted ({} frame(s) remaining).\n\
                Run `giff sync --continue` to resume, or delete .git/giff_sync_resume.json to start over.",
                state.pending_frame_ids.len()
            );
        }

        let current = backend.current_branch()?;
        let stack_id = {
            let (s, _) = store
                .find_stack_for_branch(&current)
                .ok_or_else(|| anyhow::anyhow!("branch `{}` is not in a stack", current))?;
            s.id.clone()
        };

        // Best-effort reconciliation: ask GitHub which PRs in this stack are already merged
        // (e.g. via the web UI's Merge button), prune those frames locally, and retarget every
        // affected child PR's base on GitHub. Silently skipped if no token or no GitHub remote.
        match reconcile_merged_prs(&mut store, &stack_id, &backend) {
            Ok(true) => {
                write_stack_store(&store_path, &store)?;
            }
            Ok(false) => {}
            Err(e) => {
                eprintln!("warning: skipping merged-PR check: {}", e);
                eprintln!(
                    "  → If a PR in this stack was merged via GitHub's web UI, child PR bases \
                     won't be retargeted automatically. Set GITHUB_TOKEN and re-run `giff sync`."
                );
            }
        }

        let stack = store
            .stacks
            .iter()
            .find(|s| s.id == stack_id)
            .ok_or_else(|| anyhow::anyhow!("stack disappeared during reconcile"))?;
        stack.validate()?;

        let trunk = stack.trunk.clone();
        let topo: Vec<StackFrame> = stack.topological_order().into_iter().cloned().collect();

        // Update trunk from origin (best-effort)
        let _ = backend.git_raw(&["fetch", "origin", &trunk]);
        let _ = backend.git_raw(&["rebase", &format!("origin/{}", trunk), &trunk]);

        let stack_id_str = stack.id.0.clone();
        restack(&backend, stack, &topo, &stack_id_str, &current)?;
        // The user's branch may have been pruned (its PR merged). In that case fall back to the
        // trunk so they end up somewhere sensible rather than on a stale branch.
        if backend.checkout(&current).is_err() {
            backend.checkout(&trunk)?;
            println!(
                "(your previous branch was merged — checked out {} instead)",
                trunk
            );
        }
        println!("Stack restacked successfully.");
    }

    Ok(())
}

/// Rebase each frame in `frames` onto its parent's branch (or trunk for roots), in order.
/// Saves resume state on conflict and bails.
fn restack(
    backend: &ShellGitBackend,
    stack: &Stack,
    frames: &[StackFrame],
    stack_id: &str,
    original_branch: &str,
) -> Result<()> {
    let total = frames.len();
    for (i, frame) in frames.iter().enumerate() {
        let onto = onto_branch(stack, frame)?;

        println!(
            "[{}/{}] Rebasing {} onto {}...",
            i + 1,
            total,
            frame.branch,
            onto
        );

        match backend.rebase(&frame.branch, &onto)? {
            RebaseOutcome::Clean => println!("  ✓ clean"),
            RebaseOutcome::Conflict { frame: f, hints } => {
                eprintln!("  conflict in {}", f);
                for h in &hints {
                    eprintln!("    {}", h);
                }
                let pending_frame_ids: Vec<String> =
                    frames[i + 1..].iter().map(|f| f.id.0.clone()).collect();
                save_resume(&SyncResume {
                    stack_id: stack_id.to_string(),
                    pending_frame_ids,
                    original_branch: original_branch.to_string(),
                })?;
                eprintln!();
                eprintln!("Resolve the conflicts, stage your changes, then run:");
                eprintln!("  git rebase --continue");
                eprintln!("  giff sync --continue");
                bail!("rebase conflict in frame `{}`", f);
            }
        }
    }
    Ok(())
}

fn onto_branch(stack: &Stack, frame: &StackFrame) -> Result<String> {
    Ok(match frame.parent.as_ref() {
        None => stack.trunk.clone(),
        Some(parent_id) => stack
            .frame(parent_id)
            .ok_or_else(|| anyhow::anyhow!("parent frame missing for `{}`", frame.branch))?
            .branch
            .clone(),
    })
}

/// Detect PRs that have already been merged (e.g. via GitHub's web UI), prune those frames from
/// the local stack, and update every affected child PR's `base` on GitHub.
///
/// Returns `Ok(true)` if the store was mutated and should be written back, `Ok(false)` if the
/// stack was already up-to-date, or `Err` if GitHub access failed in a way the caller should
/// surface (no token, no remote, etc. — which `run` treats as a non-fatal warning).
fn reconcile_merged_prs(
    store: &mut StackStore,
    stack_id: &StackId,
    backend: &ShellGitBackend,
) -> Result<bool> {
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
        .ok_or_else(|| anyhow::anyhow!("no GitHub token (skipping merged-PR reconcile)"))?;

    let remote_url = backend
        .git_raw(&["remote", "get-url", "origin"])
        .map_err(|_| anyhow::anyhow!("no `origin` remote (skipping merged-PR reconcile)"))?;
    let repo = parse_github_repo(&remote_url)
        .ok_or_else(|| anyhow::anyhow!("non-GitHub remote (skipping merged-PR reconcile)"))?;
    let forge = GitHubForge::new(token, repo, cfg.github.base_url);

    // 1. Identify merged frames. PR fetch failures are treated as "not merged" — better to
    //    proceed than to abort sync over a transient API hiccup — but the user must hear about
    //    each failure so they don't think we silently certified a frame as up-to-date.
    //    Fetches run in parallel against GitHub for speed; HTTP_WORKERS caps concurrency.
    let stack = store
        .stacks
        .iter()
        .find(|s| s.id == *stack_id)
        .ok_or_else(|| anyhow::anyhow!("stack not found"))?;
    let pr_jobs: Vec<(FrameId, String, u64)> = stack
        .frames
        .iter()
        .filter_map(|f| f.pr_number.map(|n| (f.id.clone(), f.branch.clone(), n)))
        .collect();
    let pr_results = crate::concurrency::parallel_map(
        pr_jobs,
        crate::concurrency::HTTP_WORKERS,
        |(fid, branch, pr_num)| (fid.clone(), branch.clone(), *pr_num, forge.get_pr(*pr_num)),
    );
    let mut merged: HashSet<FrameId> = HashSet::new();
    let mut check_failures: Vec<u64> = Vec::new();
    for (fid, branch, pr_num, res) in pr_results {
        match res {
            Ok(pr) => {
                if pr.merged {
                    merged.insert(fid);
                }
            }
            Err(e) => {
                eprintln!(
                    "warning: could not check status of PR #{} ({}): {}",
                    pr_num, branch, e
                );
                check_failures.push(pr_num);
            }
        }
    }
    if !check_failures.is_empty() {
        eprintln!(
            "  → {} PR status check(s) failed; those frames are assumed not merged. \
             Re-run `giff sync` once GitHub is reachable.",
            check_failures.len()
        );
    }
    if merged.is_empty() {
        return Ok(false);
    }

    println!(
        "detected {} merged PR(s) — pruning from stack and retargeting child PRs",
        merged.len()
    );

    // 2. Compute new parent for each remaining frame, and the resulting new base branch.
    let updates = stack.parent_updates_after_pruning(&merged);
    let trunk = stack.trunk.clone();
    let frame_branches: std::collections::HashMap<FrameId, String> = stack
        .frames
        .iter()
        .map(|f| (f.id.clone(), f.branch.clone()))
        .collect();

    // 3. Push base updates to GitHub. We don't bail on a single failure — if half the children
    //    retarget and half don't, the user's better off with the local store mutated to match
    //    what we tried to push, plus a clear instruction to re-run sync to retry.
    let mut retarget_failures: Vec<u64> = Vec::new();
    for (frame_id, new_parent) in &updates {
        let frame = stack.frame(frame_id).unwrap();
        let new_base = match new_parent {
            None => trunk.clone(),
            Some(pid) => frame_branches
                .get(pid)
                .cloned()
                .unwrap_or_else(|| trunk.clone()),
        };
        if let Some(pr_num) = frame.pr_number {
            println!("  retargeting PR #{} → {}", pr_num, new_base);
            if let Err(e) = forge.update_pr(
                pr_num,
                UpdatePrParams {
                    body: None,
                    base: Some(new_base),
                },
            ) {
                eprintln!("    warning: failed to update PR #{}: {}", pr_num, e);
                retarget_failures.push(pr_num);
            }
        }
    }
    if !retarget_failures.is_empty() {
        eprintln!(
            "  → {} PR base retarget(s) failed: {:?}",
            retarget_failures.len(),
            retarget_failures
        );
        eprintln!(
            "  → Their bases on GitHub still point at merged branches. Re-run `giff sync` to retry."
        );
    }

    // 4. Mutate the store: apply the new parents, then drop the merged frames.
    let stack_mut = store.stacks.iter_mut().find(|s| s.id == *stack_id).unwrap();
    for f in stack_mut.frames.iter_mut() {
        if let Some(np) = updates.get(&f.id) {
            f.parent = np.clone();
        }
    }
    for merged_id in &merged {
        if let Some(f) = stack_mut.frames.iter().find(|f| &f.id == merged_id) {
            println!("  pruned merged frame: {}", f.branch);
        }
    }
    stack_mut.frames.retain(|f| !merged.contains(&f.id));
    stack_mut.validate()?;

    Ok(true)
}

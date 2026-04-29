// crates/giff-cli/src/commands/sync.rs
use crate::config::{find_stack_store_path, read_stack_store};
use anyhow::{bail, Result};
use giff_git::{GitBackend, RebaseOutcome, ShellGitBackend};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Serialize, Deserialize)]
struct SyncResume {
    stack_id: String,
    resume_from_idx: usize,
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
    let store = read_stack_store(&store_path)?;
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

        let frames: Vec<_> = stack.ordered_frames().into_iter().cloned().collect();
        let total = frames.len();
        let start = state.resume_from_idx;

        println!(
            "Resuming sync from frame {}/{}: {}",
            start + 1,
            total,
            frames.get(start).map(|f| f.branch.as_str()).unwrap_or("?")
        );

        restack_frames(
            &backend,
            &frames,
            total,
            start,
            &state.original_branch,
            &state.stack_id,
        )?;
        clear_resume()?;
    } else {
        if let Some(state) = load_resume()? {
            bail!(
                "a previous sync was interrupted at frame {}.\n\
                Run `giff sync --continue` to resume, or delete .git/giff_sync_resume.json to start over.",
                state.resume_from_idx + 1
            );
        }

        let current = backend.current_branch()?;
        let (stack, _) = store
            .find_stack_for_branch(&current)
            .ok_or_else(|| anyhow::anyhow!("branch `{}` is not in a stack", current))?;

        let stack_id = stack.id.0.clone();
        let trunk = stack.trunk.clone();
        let frames: Vec<_> = stack.ordered_frames().into_iter().cloned().collect();
        let total = frames.len();

        // Update trunk from origin (best-effort)
        let _ = backend.git_raw(&["fetch", "origin", &trunk]);
        let _ = backend.git_raw(&["rebase", &format!("origin/{}", trunk), &trunk]);

        for (i, frame) in frames.iter().enumerate() {
            let onto = if i == 0 {
                trunk.clone()
            } else {
                frames[i - 1].branch.clone()
            };

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
                    save_resume(&SyncResume {
                        stack_id,
                        resume_from_idx: i + 1,
                        original_branch: current.clone(),
                    })?;
                    eprintln!();
                    eprintln!("Resolve the conflicts, stage your changes, then run:");
                    eprintln!("  git rebase --continue");
                    eprintln!("  giff sync --continue");
                    bail!("rebase conflict in frame `{}`", f);
                }
            }
        }

        backend.checkout(&current)?;
        println!("Stack restacked successfully.");
    }

    Ok(())
}

fn restack_frames(
    backend: &ShellGitBackend,
    frames: &[giff_core::StackFrame],
    total: usize,
    start: usize,
    original_branch: &str,
    stack_id: &str,
) -> Result<()> {
    // Load trunk for the stack so we can compute `onto` for any frame index.
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let trunk = frames
        .first()
        .and_then(|f| store.find_stack_for_branch(&f.branch))
        .map(|(s, _)| s.trunk.clone())
        .unwrap_or_else(|| "main".into());

    for i in start..frames.len() {
        let onto = if i == 0 {
            trunk.clone()
        } else {
            frames[i - 1].branch.clone()
        };

        println!(
            "[{}/{}] Rebasing {} onto {}...",
            i + 1,
            total,
            frames[i].branch,
            onto
        );

        match backend.rebase(&frames[i].branch, &onto)? {
            RebaseOutcome::Clean => println!("  ✓ clean"),
            RebaseOutcome::Conflict { frame: f, hints } => {
                eprintln!("  conflict in {}", f);
                for h in &hints {
                    eprintln!("    {}", h);
                }
                save_resume(&SyncResume {
                    stack_id: stack_id.to_string(),
                    resume_from_idx: i + 1,
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

    backend.checkout(original_branch)?;
    println!("Stack restacked successfully.");
    Ok(())
}

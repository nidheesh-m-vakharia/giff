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

    let template_body = if cfg.defaults.pr_template.is_empty() {
        None
    } else {
        let path = std::path::PathBuf::from(&cfg.defaults.pr_template);
        Some(
            std::fs::read_to_string(&path)
                .with_context(|| format!("reading pr_template at {}", path.display()))?,
        )
    };

    let stack = store
        .stacks
        .iter()
        .find(|s| s.id == stack_id)
        .unwrap()
        .clone();
    stack.validate()?;
    let total = stack.frames.len();

    // Topological order guarantees parents are listed before children, so each PR can reliably
    // target its parent's branch.
    let topo: Vec<_> = stack.topological_order().into_iter().cloned().collect();

    // Phase 1 — push every branch in a SINGLE `git push` invocation so SSH only handshakes
    // once for the whole stack (instead of N times). git push accepts multiple refspecs, and
    // --force-with-lease is the safe variant of `--force` (refuses to clobber if the remote
    // moved out from under us).
    {
        let mut args: Vec<&str> = vec!["push", "--force-with-lease", "origin"];
        let refspecs: Vec<String> = topo
            .iter()
            .map(|f| format!("{0}:{0}", f.branch))
            .collect();
        for r in &refspecs {
            args.push(r.as_str());
        }
        backend.git_raw(&args)?;
    }

    // Phase 2 — build the per-frame API jobs. Each PR's `base` references its parent's branch
    // *name* (not number), so once Phase 1 finished, every PR call is independent of the others
    // and we can fan them out across HTTP_WORKERS threads.
    struct Job {
        frame_id: giff_core::FrameId,
        branch: String,
        existing_pr: Option<u64>,
        params_create: CreatePrParams,
        params_update: UpdatePrParams,
    }

    let jobs: Vec<Job> = topo
        .iter()
        .enumerate()
        .map(|(i, frame)| -> Result<Job> {
            let position = i + 1;
            let base = match frame.parent.as_ref() {
                None => stack.trunk.clone(),
                Some(parent_id) => stack
                    .frame(parent_id)
                    .ok_or_else(|| anyhow::anyhow!("parent frame missing for `{}`", frame.branch))?
                    .branch
                    .clone(),
            };
            let meta = RemoteStackMeta {
                stack_id: stack.id.clone(),
                frame_id: frame.id.clone(),
                parent_frame_id: frame.parent.clone(),
                position,
                total,
            };
            let stack_line = format!("Part {}/{} of stack `{}`.", position, total, stack.name);
            let body = match &template_body {
                Some(t) => format!("{}\n\n{}\n\n{}", t.trim_end(), stack_line, meta.to_pr_block()),
                None => format!("{}\n\n{}", stack_line, meta.to_pr_block()),
            };
            Ok(Job {
                frame_id: frame.id.clone(),
                branch: frame.branch.clone(),
                existing_pr: frame.pr_number,
                params_create: CreatePrParams {
                    title: frame.branch.clone(),
                    body: body.clone(),
                    head: frame.branch.clone(),
                    base: base.clone(),
                    draft: cfg.defaults.draft_prs,
                },
                params_update: UpdatePrParams {
                    body: Some(body),
                    base: Some(base),
                },
            })
        })
        .collect::<Result<_>>()?;

    // Phase 3 — fire the API calls in parallel. Result tuple = (frame_id, branch, outcome).
    let outcomes = crate::concurrency::parallel_map(
        jobs,
        crate::concurrency::HTTP_WORKERS,
        |job| -> (giff_core::FrameId, String, Result<u64, String>) {
            let result = match job.existing_pr {
                Some(existing) => forge
                    .update_pr(existing, job.params_update.clone())
                    .map(|_| existing)
                    .map_err(|e| e.to_string()),
                None => forge
                    .create_pr(job.params_create.clone())
                    .map(|pr| pr.number)
                    .map_err(|e| e.to_string()),
            };
            (job.frame_id.clone(), job.branch.clone(), result)
        },
    );

    // Phase 4 — apply successes to the store, summarise failures. We always write whatever
    // succeeded so a partial failure can be retried by re-running `giff push`.
    let mut failures: Vec<(String, String)> = Vec::new();
    for (frame_id, branch, result) in &outcomes {
        match result {
            Ok(pr_number) => {
                let s = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
                let f = s.frames.iter_mut().find(|f| f.id == *frame_id).unwrap();
                f.pr_number = Some(*pr_number);
                println!("  {} → PR #{}", branch, pr_number);
            }
            Err(e) => {
                eprintln!("  {} → FAILED: {}", branch, e);
                failures.push((branch.clone(), e.clone()));
            }
        }
    }

    write_stack_store(&store_path, &store)?;

    if !failures.is_empty() {
        eprintln!();
        eprintln!(
            "warning: {} of {} PR operations failed:",
            failures.len(),
            outcomes.len()
        );
        for (branch, err) in &failures {
            eprintln!("  • {}: {}", branch, err);
        }
        eprintln!("Re-run `giff push` to retry.");
        anyhow::bail!("partial push failure");
    }

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

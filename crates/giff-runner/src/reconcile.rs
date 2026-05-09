//! Shared post-event logic. Both the polling worker and the webhook handler funnel through
//! `reconcile_after_upsert` so behaviour is identical regardless of where the signal came
//! from. Idempotent — duplicate events from webhook redelivery + a polling tick observing
//! the same merge are no-ops on the second pass.

use crate::config::{Config, RepoConfig};
use crate::db::Db;
use crate::grouping::{self, Stack};
use crate::retry::{self, JobKind};
use anyhow::{Context, Result};
use giff_core::{FrameId, RemoteStackMeta};
use giff_github::{ForgeBackend, GitHubForge, UpdatePrParams};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Build a forge for a given repo using the config's token + base URL.
pub fn forge_for(cfg: &Config, repo_slug: &str) -> GitHubForge {
    GitHubForge::new(
        cfg.github_token.clone(),
        repo_slug.to_string(),
        cfg.github_base_url.clone(),
    )
}

/// Fetch a PR and write its snapshot to the DB. Used by webhook handlers when the payload
/// references a PR by number — we re-fetch from GitHub rather than trusting payload bodies
/// directly so a malicious or malformed delivery can't poison state.
pub async fn refresh_pull(db: &Db, cfg: &Config, repo: &str, number: u64) -> Result<()> {
    let forge = forge_for(cfg, repo);
    let repo_owned = repo.to_string();
    let pr = tokio::task::spawn_blocking(move || forge.get_pr(number))
        .await
        .context("blocking task panicked")?
        .with_context(|| format!("fetching PR #{} from {}", number, repo_owned))?;
    db.upsert_pull(repo.to_string(), pr, now_secs()).await?;
    Ok(())
}

/// After upserting a PR snapshot, walk the affected stack and:
///   1. Retarget any open child PR whose parent was just merged (so the child points at
///      whatever the parent's parent was — or trunk if the parent was a root).
///   2. If `RepoConfig::auto_merge` is on, merge any single-root stack whose root is
///      mergeable + has approving review (delegated to a future call site — for v1 we
///      log only, since the approval check needs another endpoint).
///
/// Always idempotent. Re-running with no state change is a no-op.
pub async fn reconcile_repo(db: Arc<Db>, cfg: Arc<Config>, repo: &str) -> Result<()> {
    let repo_cfg = cfg.find_repo(repo).cloned().unwrap_or(RepoConfig {
        slug: repo.to_string(),
        auto_merge: false,
        merge_method: "merge".into(),
        webhook_secret: None,
    });

    let pulls = db.list_pulls(repo.to_string()).await?;
    let grouping = grouping::group(pulls);

    let forge = forge_for(&cfg, repo);

    for stack in &grouping.stacks {
        retarget_children_of_merged(&db, &forge, repo, stack).await?;
    }

    if repo_cfg.auto_merge {
        for stack in &grouping.stacks {
            try_auto_merge(&db, &forge, repo, &repo_cfg, stack).await?;
        }
    }

    Ok(())
}

/// For a stack, identify frames whose PR is merged and retarget their children's PR bases on
/// GitHub. We re-derive each child's correct base by walking up its parent chain past any
/// merged ancestors.
async fn retarget_children_of_merged(
    db: &Db,
    forge: &GitHubForge,
    repo: &str,
    stack: &Stack,
) -> Result<()> {
    let merged: HashSet<FrameId> = stack
        .frames
        .iter()
        .filter(|f| f.pr.merged)
        .map(|f| f.meta.frame_id.clone())
        .collect();
    if merged.is_empty() {
        return Ok(());
    }

    // Build the same "new parent for each non-merged frame" map giff-core's
    // `parent_updates_after_pruning` produces locally. Inline here because grouping uses a
    // different shape than core's `Stack`. The algorithm is the same.
    let by_id: std::collections::HashMap<FrameId, &RemoteStackMeta> = stack
        .frames
        .iter()
        .map(|f| (f.meta.frame_id.clone(), &f.meta))
        .collect();
    let head_ref_by_id: std::collections::HashMap<FrameId, String> = stack
        .frames
        .iter()
        .map(|f| (f.meta.frame_id.clone(), f.pr.head_ref.clone()))
        .collect();

    for f in &stack.frames {
        if f.pr.merged || f.pr.state != "open" {
            continue;
        }
        let mut new_parent = f.meta.parent_frame_id.clone();
        while let Some(ref pid) = new_parent {
            if merged.contains(pid) {
                new_parent = by_id.get(pid).and_then(|m| m.parent_frame_id.clone());
            } else {
                break;
            }
        }
        if new_parent != f.meta.parent_frame_id {
            // Find the new base branch name. None ⇒ trunk; we don't store the trunk name
            // for the stack here, so we ask GitHub by reading the merged frame's base_ref
            // (which is the trunk for stack roots, the parent's branch otherwise).
            let trunk_guess = stack
                .frames
                .iter()
                .find(|x| x.meta.parent_frame_id.is_none())
                .map(|x| x.pr.base_ref.clone())
                .unwrap_or_else(|| "main".into());
            let new_base = match &new_parent {
                None => trunk_guess,
                Some(pid) => head_ref_by_id.get(pid).cloned().unwrap_or_default(),
            };

            let pr_num = f.pr.number;
            let new_base_clone = new_base.clone();
            let forge = forge.clone();
            let res = tokio::task::spawn_blocking(move || {
                forge.update_pr(
                    pr_num,
                    UpdatePrParams {
                        body: None,
                        base: Some(new_base_clone),
                    },
                )
            })
            .await
            .context("blocking task panicked")?;

            match res {
                Ok(_) => {
                    db.record_event(
                        repo.to_string(),
                        Some(pr_num),
                        "retargeted".into(),
                        Some(format!("base → {}", new_base)),
                        now_secs(),
                    )
                    .await?;
                    tracing::info!(
                        repo = repo,
                        pr = pr_num,
                        new_base = %new_base,
                        "retargeted PR base"
                    );
                }
                Err(e) => {
                    db.record_event(
                        repo.to_string(),
                        Some(pr_num),
                        "error".into(),
                        Some(format!("retarget failed: {}", e)),
                        now_secs(),
                    )
                    .await?;
                    let kind = JobKind::RetargetBase { base: new_base.clone() };
                    if let Err(qe) = retry::enqueue(db, repo, pr_num, &kind).await {
                        tracing::error!(error = %qe, "could not enqueue retry job");
                    }
                    tracing::warn!(
                        repo = repo,
                        pr = pr_num,
                        error = %e,
                        "retarget failed; queued for retry"
                    );
                }
            }
        }
    }
    Ok(())
}

/// Auto-merge gate. Conservative for v1: merge a stack root iff its PR is open + draft is
/// false + GitHub reports the PR mergeable. Approving-review enforcement is deliberately
/// deferred — it requires another endpoint and a UX decision (which review states count?).
/// For now if you turn on auto_merge you're saying "trust me, my branch protection rules
/// already enforce the review requirement" — typical for teams that already use required
/// reviews on the trunk.
async fn try_auto_merge(
    db: &Db,
    forge: &GitHubForge,
    repo: &str,
    repo_cfg: &RepoConfig,
    stack: &Stack,
) -> Result<()> {
    if stack.roots.len() != 1 {
        return Ok(()); // multi-root: ambiguous, skip.
    }
    let root_pr_number = stack.roots[0].pr_number;
    let frame = match stack.frames.iter().find(|f| f.pr.number == root_pr_number) {
        Some(f) => f,
        None => return Ok(()),
    };
    if frame.pr.merged || frame.pr.state != "open" || frame.pr.draft {
        return Ok(());
    }

    let forge_clone = forge.clone();
    let status = tokio::task::spawn_blocking(move || forge_clone.pr_status(root_pr_number))
        .await
        .context("blocking task panicked")?;
    let status = match status {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(repo = repo, pr = root_pr_number, error = %e, "auto-merge: pr_status failed");
            return Ok(());
        }
    };
    if status.draft || status.mergeable != Some(true) {
        return Ok(());
    }

    let method = repo_cfg.merge_method.clone();
    let forge_clone = forge.clone();
    let res = tokio::task::spawn_blocking(move || forge_clone.merge_pr(root_pr_number, &method))
        .await
        .context("blocking task panicked")?;
    match res {
        Ok(_) => {
            db.record_event(
                repo.to_string(),
                Some(root_pr_number),
                "auto_merged".into(),
                Some(format!("via {}", repo_cfg.merge_method)),
                now_secs(),
            )
            .await?;
            tracing::info!(
                repo = repo,
                pr = root_pr_number,
                method = %repo_cfg.merge_method,
                "auto-merged stack root"
            );
        }
        Err(e) => {
            db.record_event(
                repo.to_string(),
                Some(root_pr_number),
                "error".into(),
                Some(format!("auto-merge failed: {}", e)),
                now_secs(),
            )
            .await?;
            let kind = JobKind::AutoMerge {
                method: repo_cfg.merge_method.clone(),
            };
            if let Err(qe) = retry::enqueue(db, repo, root_pr_number, &kind).await {
                tracing::error!(error = %qe, "could not enqueue retry job");
            }
            tracing::warn!(
                repo = repo,
                pr = root_pr_number,
                error = %e,
                "auto-merge failed; queued for retry"
            );
        }
    }
    Ok(())
}

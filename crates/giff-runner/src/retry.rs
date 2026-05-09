//! Retry queue. Failed `update_pr` and `merge_pr` calls land in `retry_jobs` (SQLite) and
//! get re-tried on a backoff schedule by a dedicated tokio task. Idempotent throughout —
//! re-deciding the same job in `reconcile.rs` no-ops thanks to the natural-key UNIQUE
//! constraint, and re-executing a now-stale action against GitHub typically returns 200
//! (already in target state) or a clear error that we then count toward abandonment.
//!
//! Backoff schedule (attempts → delay before next try):
//!   0 → 30s, 1 → 1m, 2 → 5m, 3 → 15m, 4 → 30m, 5 → 1h, 6 → 4h, then abandon.
//! ~6.5 hours of trying before giving up. The audit trail (success / abandon) lives in
//! the `events` table.

use crate::config::Config;
use crate::db::{Db, RetryRow};
use crate::reconcile::{forge_for, now_secs};
use anyhow::{Context, Result};
use giff_github::{ForgeBackend, UpdatePrParams};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Tick cadence for the retry worker. Fast enough that even the shortest backoff (30s)
/// fires roughly on time without wall-clock skew accumulating.
const TICK_SECS: u64 = 10;
/// Per-tick batch cap. With 10s ticks and conservative API calls, 50 keeps the worker
/// from accidentally hammering GitHub on a recovery wave.
const BATCH_LIMIT: usize = 50;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JobKind {
    /// Set a child PR's `base` to the given branch on GitHub.
    RetargetBase { base: String },
    /// Merge the bottom (root) PR of a stack via the given method.
    AutoMerge { method: String },
}

impl JobKind {
    pub fn discriminant(&self) -> &'static str {
        match self {
            JobKind::RetargetBase { .. } => "retarget_base",
            JobKind::AutoMerge { .. } => "auto_merge",
        }
    }
}

/// Compute the delay (seconds) before the next attempt given the current `attempts` count.
/// `None` means "abandon — don't reschedule."
pub fn next_delay_secs(attempts: i64) -> Option<i64> {
    match attempts {
        0 => Some(30),
        1 => Some(60),
        2 => Some(5 * 60),
        3 => Some(15 * 60),
        4 => Some(30 * 60),
        5 => Some(60 * 60),
        6 => Some(4 * 60 * 60),
        _ => None,
    }
}

/// Enqueue a job to be retried after `next_delay_secs(0)` (the initial backoff).
/// The natural-key UNIQUE on the table de-dupes against any pending job for the same
/// (kind, repo, pr_number, payload).
pub async fn enqueue(db: &Db, repo: &str, pr_number: u64, kind: &JobKind) -> Result<()> {
    let now = now_secs();
    let initial_delay = next_delay_secs(0).unwrap_or(60);
    let payload = serde_json::to_string(kind)?;
    db.enqueue_retry(
        kind.discriminant().to_string(),
        repo.to_string(),
        pr_number,
        payload,
        now + initial_delay,
        now,
    )
    .await
}

/// Spawn the retry worker. Returns a Notify the rest of the system can poke to wake the
/// worker early (e.g. from `POST /sync` to flush retries on demand).
pub fn spawn(db: Arc<Db>, cfg: Arc<Config>) -> Arc<Notify> {
    let trigger = Arc::new(Notify::new());
    let trigger_for_task = trigger.clone();
    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = trigger_for_task.notified() => {}
                _ = tokio::time::sleep(Duration::from_secs(TICK_SECS)) => {}
            }
            if let Err(e) = run_due_jobs(&db, &cfg).await {
                tracing::error!(error = %e, "retry worker tick failed at top level");
            }
        }
    });
    trigger
}

async fn run_due_jobs(db: &Arc<Db>, cfg: &Arc<Config>) -> Result<()> {
    let due = db.claim_ready_retries(now_secs(), BATCH_LIMIT).await?;
    if due.is_empty() {
        return Ok(());
    }
    tracing::debug!(count = due.len(), "running due retry jobs");
    for job in due {
        if let Err(e) = run_job(db, cfg, &job).await {
            tracing::error!(job_id = job.id, error = %e, "retry job dispatch failed");
        }
    }
    Ok(())
}

async fn run_job(db: &Db, cfg: &Config, job: &RetryRow) -> Result<()> {
    let kind: JobKind = serde_json::from_str(&job.payload)
        .with_context(|| format!("decoding retry job #{} payload", job.id))?;
    let result = execute(cfg, &job.repo, job.pr_number, &kind).await;
    match result {
        Ok(_) => {
            db.complete_retry(job.id).await?;
            db.record_event(
                job.repo.clone(),
                Some(job.pr_number),
                "retry_succeeded".into(),
                Some(format!(
                    "{} after {} attempt(s)",
                    job.kind,
                    job.attempts + 1
                )),
                now_secs(),
            )
            .await?;
            tracing::info!(
                job_id = job.id,
                kind = %job.kind,
                repo = %job.repo,
                pr = job.pr_number,
                attempts = job.attempts + 1,
                "retry succeeded"
            );
            Ok(())
        }
        Err(e) => {
            let next_attempts = job.attempts + 1;
            match next_delay_secs(next_attempts) {
                Some(delay) => {
                    db.bump_retry(job.id, format!("{}", e), now_secs() + delay)
                        .await?;
                    tracing::warn!(
                        job_id = job.id,
                        kind = %job.kind,
                        repo = %job.repo,
                        pr = job.pr_number,
                        attempts = next_attempts,
                        next_in_secs = delay,
                        error = %e,
                        "retry failed; rescheduled"
                    );
                }
                None => {
                    db.complete_retry(job.id).await?;
                    db.record_event(
                        job.repo.clone(),
                        Some(job.pr_number),
                        "abandoned".into(),
                        Some(format!(
                            "{} after {} attempts: {}",
                            job.kind, next_attempts, e
                        )),
                        now_secs(),
                    )
                    .await?;
                    tracing::error!(
                        job_id = job.id,
                        kind = %job.kind,
                        repo = %job.repo,
                        pr = job.pr_number,
                        attempts = next_attempts,
                        error = %e,
                        "retry abandoned"
                    );
                }
            }
            Ok(())
        }
    }
}

async fn execute(cfg: &Config, repo: &str, pr_number: u64, kind: &JobKind) -> Result<()> {
    let forge = forge_for(cfg, repo);
    match kind {
        JobKind::RetargetBase { base } => {
            let base = base.clone();
            tokio::task::spawn_blocking(move || {
                forge.update_pr(
                    pr_number,
                    UpdatePrParams {
                        body: None,
                        base: Some(base),
                    },
                )
            })
            .await
            .context("blocking task panicked")?
            .context("update_pr failed")?;
        }
        JobKind::AutoMerge { method } => {
            let method = method.clone();
            tokio::task::spawn_blocking(move || forge.merge_pr(pr_number, &method))
                .await
                .context("blocking task panicked")?
                .context("merge_pr failed")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_schedule_terminates() {
        // Sanity-check the schedule: must reach `None` so jobs eventually abandon.
        let mut a = 0;
        let mut total = 0;
        while let Some(d) = next_delay_secs(a) {
            total += d;
            a += 1;
            if a > 100 {
                panic!("backoff schedule didn't terminate");
            }
        }
        // Total budget is roughly 6h 21m. A wide acceptable range so future tweaks don't
        // break this assertion.
        assert!(total > 60 * 60, "schedule too short: {}s", total);
        assert!(total < 24 * 60 * 60, "schedule too long: {}s", total);
    }

    #[test]
    fn jobkind_round_trips_through_json() {
        let k = JobKind::RetargetBase { base: "main".into() };
        let s = serde_json::to_string(&k).unwrap();
        let back: JobKind = serde_json::from_str(&s).unwrap();
        match back {
            JobKind::RetargetBase { base } => assert_eq!(base, "main"),
            _ => panic!("wrong variant"),
        }
    }
}

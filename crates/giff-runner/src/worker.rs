//! Polling worker. Long-interval safety net for missed webhooks (or polling-only mode for
//! repos without `webhook_secret` configured). Same code path as the webhook handler — both
//! call `reconcile_repo` after upserting PR snapshots.

use crate::config::Config;
use crate::db::Db;
use crate::reconcile::{forge_for, now_secs, reconcile_repo};
use anyhow::Result;
use giff_github::ForgeBackend;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;

/// Spawn the polling task. Returns a handle to a `Notify` that can be triggered to force an
/// immediate sync (used by the `POST /sync` API route and on startup).
pub fn spawn(db: Arc<Db>, cfg: Arc<Config>) -> Arc<Notify> {
    let trigger = Arc::new(Notify::new());
    let trigger_for_task = trigger.clone();
    tokio::spawn(async move {
        // Kick the first cycle immediately so the DB has data right after startup.
        trigger_for_task.notify_one();
        loop {
            tokio::select! {
                _ = trigger_for_task.notified() => {}
                _ = tokio::time::sleep(Duration::from_secs(cfg.poll_seconds)) => {}
            }
            if let Err(e) = poll_once(&db, &cfg).await {
                tracing::error!(error = %e, "poll cycle errored at top level");
            }
        }
    });
    trigger
}

async fn poll_once(db: &Arc<Db>, cfg: &Arc<Config>) -> Result<()> {
    for repo_cfg in &cfg.repos {
        let slug = repo_cfg.slug.clone();
        db.upsert_repo(slug.clone()).await?;

        match sync_repo(db.clone(), cfg.clone(), &slug).await {
            Ok(n) => {
                db.mark_repo_synced(slug.clone(), now_secs()).await?;
                tracing::info!(repo = %slug, pulls = n, "polled");
            }
            Err(e) => {
                let msg = format!("{:#}", e);
                db.mark_repo_error(slug.clone(), msg.clone()).await?;
                db.record_event(
                    slug.clone(),
                    None,
                    "error".into(),
                    Some(msg.clone()),
                    now_secs(),
                )
                .await?;
                tracing::warn!(repo = %slug, error = %msg, "poll failed");
            }
        }
    }
    Ok(())
}

async fn sync_repo(db: Arc<Db>, cfg: Arc<Config>, repo: &str) -> Result<usize> {
    let forge = forge_for(&cfg, repo);
    let pulls = tokio::task::spawn_blocking(move || forge.list_open_pulls()).await??;
    let count = pulls.len();
    let now = now_secs();
    for pr in pulls {
        db.upsert_pull(repo.to_string(), pr, now).await?;
    }
    reconcile_repo(db, cfg, repo).await?;
    Ok(count)
}

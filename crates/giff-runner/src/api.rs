//! Read-only HTTP API: health, repo status, reconstructed stacks, recent events, and an
//! immediate-sync trigger. JSON responses throughout. Authentication is intentionally
//! absent for v1 — the runner is single-tenant and expected to live behind whatever auth
//! layer the operator already runs (Cloudflare Access, Tailscale Funnel ACLs, basic-auth
//! reverse proxy, etc.).

use crate::config::Config;
use crate::db::Db;
use crate::grouping::{self, GroupedStacks, Stack};
use crate::webhook::{self, WebhookState};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json};
use axum::routing::{get, post};
use axum::Router;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone)]
pub struct ApiState {
    pub db: Arc<Db>,
    pub cfg: Arc<Config>,
    pub poll_trigger: Arc<Notify>,
    pub retry_trigger: Arc<Notify>,
}

/// Build the public router: API + webhook routes, each bound to their own state. The
/// webhook handler lives in its own sub-router with `WebhookState` so it doesn't carry
/// the API-only triggers it doesn't need.
pub fn build_router(
    db: Arc<Db>,
    cfg: Arc<Config>,
    poll_trigger: Arc<Notify>,
    retry_trigger: Arc<Notify>,
) -> Router {
    let api_state = ApiState {
        db: db.clone(),
        cfg: cfg.clone(),
        poll_trigger,
        retry_trigger,
    };
    let webhook_state = WebhookState { db, cfg };

    let api = Router::new()
        .route("/healthz", get(healthz))
        .route("/repos", get(list_repos))
        .route("/stacks", get(list_stacks))
        .route("/stacks/:id", get(get_stack))
        .route("/events", get(list_events))
        .route("/retry-queue", get(list_retry_queue))
        .route("/sync", post(trigger_sync))
        .with_state(api_state);

    let webhooks = Router::new()
        .route("/webhook/github", post(webhook::handle))
        .with_state(webhook_state);

    api.merge(webhooks)
}

async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

#[derive(Serialize)]
struct RepoView {
    slug: String,
    last_synced_at: Option<i64>,
    last_error: Option<String>,
    auto_merge: bool,
    merge_method: String,
    webhooks_configured: bool,
}

async fn list_repos(State(s): State<ApiState>) -> impl IntoResponse {
    let status = match s.db.list_repo_status().await {
        Ok(v) => v,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)).into_response();
        }
    };
    let view: Vec<RepoView> = s
        .cfg
        .repos
        .iter()
        .map(|r| {
            let st = status.iter().find(|x| x.slug == r.slug);
            RepoView {
                slug: r.slug.clone(),
                last_synced_at: st.and_then(|x| x.last_synced_at),
                last_error: st.and_then(|x| x.last_error.clone()),
                auto_merge: r.auto_merge,
                merge_method: r.merge_method.clone(),
                webhooks_configured: r.webhook_secret.is_some(),
            }
        })
        .collect();
    Json(view).into_response()
}

async fn list_stacks(State(s): State<ApiState>) -> impl IntoResponse {
    let pulls = match s.db.list_all_pulls().await {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)).into_response(),
    };
    let grouped: GroupedStacks = grouping::group(pulls);
    Json(grouped).into_response()
}

async fn get_stack(
    State(s): State<ApiState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let pulls = match s.db.list_all_pulls().await {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)).into_response(),
    };
    let grouped = grouping::group(pulls);
    match grouped.stacks.into_iter().find(|x: &Stack| x.id == id) {
        Some(stack) => Json(stack).into_response(),
        None => (StatusCode::NOT_FOUND, "stack not found").into_response(),
    }
}

#[derive(Deserialize)]
struct EventQuery {
    since: Option<i64>,
    limit: Option<usize>,
}

async fn list_events(
    State(s): State<ApiState>,
    Query(q): Query<EventQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(100).min(1000);
    let events = match s.db.list_events(q.since, limit).await {
        Ok(v) => v,
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)).into_response(),
    };
    #[derive(Serialize)]
    struct EventView {
        id: i64,
        repo: String,
        pr_number: Option<u64>,
        kind: String,
        detail: Option<String>,
        at: i64,
    }
    let view: Vec<EventView> = events
        .into_iter()
        .map(|e| EventView {
            id: e.id,
            repo: e.repo,
            pr_number: e.pr_number,
            kind: e.kind,
            detail: e.detail,
            at: e.at,
        })
        .collect();
    Json(view).into_response()
}

async fn trigger_sync(State(s): State<ApiState>) -> impl IntoResponse {
    // Wake both workers — a manual /sync should also flush ready retries.
    s.poll_trigger.notify_one();
    s.retry_trigger.notify_one();
    (StatusCode::ACCEPTED, "scheduled")
}

#[derive(Deserialize)]
struct RetryQueueQuery {
    limit: Option<usize>,
}

async fn list_retry_queue(
    State(s): State<ApiState>,
    Query(q): Query<RetryQueueQuery>,
) -> impl IntoResponse {
    let limit = q.limit.unwrap_or(200).min(1000);
    match s.db.list_retry_jobs(limit).await {
        Ok(rows) => Json(rows).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", e)).into_response(),
    }
}

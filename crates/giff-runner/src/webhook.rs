//! GitHub webhook receiver. HMAC-SHA256 signature verified against the per-repo
//! `webhook_secret` in config. Supported events:
//!
//! * `pull_request`         — opened, edited, synchronize, closed (with merged=true)
//! * `pull_request_review`  — submitted (used by future approval-gated auto-merge)
//! * `ping`                 — GitHub's "is this URL alive" smoke test
//!
//! Other events get a 200 with no work — webhook configs in GitHub are sometimes overly
//! broad, and dropping unknowns silently keeps logs quiet.

use crate::config::Config;
use crate::db::Db;
use crate::reconcile::{now_secs, reconcile_repo, refresh_pull};
use anyhow::Result;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::Sha256;
use std::sync::Arc;

#[derive(Clone)]
pub struct WebhookState {
    pub db: Arc<Db>,
    pub cfg: Arc<Config>,
}

// `repository` is matched only so the deserializer enforces the field's presence — the slug
// itself was already pulled out by `extract_repo_slug` before signature verification.
#[derive(Deserialize)]
struct PullRequestEnvelope {
    action: String,
    #[allow(dead_code)]
    repository: Repository,
    #[serde(default)]
    pull_request: Option<PullRef>,
}

#[derive(Deserialize)]
struct PullReviewEnvelope {
    #[allow(dead_code)]
    repository: Repository,
    #[serde(default)]
    pull_request: Option<PullRef>,
}

#[derive(Deserialize)]
struct Repository {
    full_name: String,
}

#[derive(Deserialize)]
struct PullRef {
    number: u64,
}

pub async fn handle(
    State(state): State<WebhookState>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let event = headers
        .get("X-GitHub-Event")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let signature = headers
        .get("X-Hub-Signature-256")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    if event == "ping" {
        return (StatusCode::OK, "pong").into_response();
    }

    // Identify the repo from the payload first so we can look up its secret.
    let repo_slug = match extract_repo_slug(&body) {
        Some(s) => s,
        None => return (StatusCode::BAD_REQUEST, "no repo in payload").into_response(),
    };

    let repo_cfg = match state.cfg.find_repo(&repo_slug) {
        Some(r) => r,
        None => {
            tracing::warn!(repo = %repo_slug, "webhook for un-tracked repo, ignoring");
            return (StatusCode::OK, "untracked repo").into_response();
        }
    };
    let secret = match repo_cfg.webhook_secret.as_ref() {
        Some(s) if !s.is_empty() => s,
        _ => {
            tracing::warn!(repo = %repo_slug, "webhook arrived but no secret configured for this repo");
            return (StatusCode::FORBIDDEN, "webhook_secret not configured").into_response();
        }
    };
    if !verify_signature(secret, &signature, &body) {
        return (StatusCode::UNAUTHORIZED, "bad signature").into_response();
    }

    // Signature OK. Dispatch on event type.
    let result = match event.as_str() {
        "pull_request" => handle_pull_request(&state, &repo_slug, &body).await,
        "pull_request_review" => handle_pull_review(&state, &repo_slug, &body).await,
        other => {
            tracing::debug!(event = %other, "ignoring webhook event");
            Ok(())
        }
    };

    match result {
        Ok(_) => (StatusCode::OK, "ok").into_response(),
        Err(e) => {
            tracing::error!(error = %e, "webhook handler errored");
            (StatusCode::INTERNAL_SERVER_ERROR, "handler error").into_response()
        }
    }
}

async fn handle_pull_request(state: &WebhookState, repo: &str, body: &[u8]) -> Result<()> {
    let env: PullRequestEnvelope = serde_json::from_slice(body)?;
    let pr_num = env.pull_request.as_ref().map(|p| p.number);

    if let Some(num) = pr_num {
        // Re-fetch from GitHub rather than trusting the payload directly. Pays one extra
        // request per event; gains us authoritative data even if the payload was tampered
        // with after a signature-stripping middlebox or a future GitHub schema change.
        refresh_pull(&state.db, &state.cfg, repo, num).await?;
        state
            .db
            .record_event(
                repo.to_string(),
                Some(num),
                format!("pr_{}", env.action),
                None,
                now_secs(),
            )
            .await?;
    }

    // Any pull_request action might have changed the stack shape; reconcile.
    reconcile_repo(state.db.clone(), state.cfg.clone(), repo).await?;
    Ok(())
}

async fn handle_pull_review(state: &WebhookState, repo: &str, body: &[u8]) -> Result<()> {
    let env: PullReviewEnvelope = serde_json::from_slice(body)?;
    if let Some(p) = env.pull_request {
        refresh_pull(&state.db, &state.cfg, repo, p.number).await?;
        state
            .db
            .record_event(
                repo.to_string(),
                Some(p.number),
                "review_submitted".into(),
                None,
                now_secs(),
            )
            .await?;
    }
    reconcile_repo(state.db.clone(), state.cfg.clone(), repo).await?;
    Ok(())
}

fn extract_repo_slug(body: &[u8]) -> Option<String> {
    #[derive(Deserialize)]
    struct Wrap {
        repository: Repository,
    }
    let w: Wrap = serde_json::from_slice(body).ok()?;
    Some(w.repository.full_name)
}

fn verify_signature(secret: &str, signature: &str, body: &[u8]) -> bool {
    // GitHub format: "sha256=<hex>"
    let Some(hex_sig) = signature.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(expected) = hex::decode(hex_sig) else {
        return false;
    };
    let mut mac = match Hmac::<Sha256>::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(body);
    let got = mac.finalize().into_bytes();
    // Constant-time comparison to avoid timing oracles. `subtle::ConstantTimeEq` is the
    // canonical primitive here.
    use subtle::ConstantTimeEq;
    got.as_slice().ct_eq(expected.as_slice()).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signature_round_trip() {
        let secret = "abc";
        let body = b"hello";
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        let hex_sig = format!("sha256={}", hex::encode(mac.finalize().into_bytes()));
        assert!(verify_signature(secret, &hex_sig, body));
        assert!(!verify_signature(secret, &hex_sig, b"tampered"));
        assert!(!verify_signature("wrong", &hex_sig, body));
        assert!(!verify_signature(secret, "sha256=deadbeef", body));
        assert!(!verify_signature(secret, "no-prefix", body));
    }

    #[test]
    fn extracts_repo_slug() {
        let body = br#"{"repository":{"full_name":"o/r"},"action":"opened"}"#;
        assert_eq!(extract_repo_slug(body).unwrap(), "o/r");
    }
}

use giff_core::GiffError;
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct CreatePrParams {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
    pub draft: bool,
}

#[derive(Debug, Clone)]
pub struct UpdatePrParams {
    pub body: Option<String>,
    pub base: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct BranchRef {
    #[serde(rename = "ref", default)]
    pub r#ref: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub html_url: String,
    pub state: String,
    /// Only present in detail responses (`GET /pulls/:n`); list endpoints don't include it,
    /// so we default to false rather than fail to deserialize there.
    #[serde(default)]
    pub merged: bool,
    /// Set on list and detail responses; runner uses this in its dashboard.
    #[serde(default)]
    pub title: String,
    /// Carries the embedded ```giff metadata block (parsed by giff-core).
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub head: BranchRef,
    #[serde(default)]
    pub base: BranchRef,
    #[serde(default)]
    pub draft: bool,
    /// ISO 8601 timestamp; the runner uses this to break ties when reconciling.
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrStatus {
    pub mergeable: Option<bool>,
    pub draft: bool,
}

pub trait ForgeBackend {
    fn create_pr(&self, params: CreatePrParams) -> Result<PullRequest, GiffError>;
    fn update_pr(&self, number: u64, params: UpdatePrParams) -> Result<PullRequest, GiffError>;
    fn get_pr(&self, number: u64) -> Result<PullRequest, GiffError>;
    fn pr_status(&self, number: u64) -> Result<PrStatus, GiffError>;
    /// Merge a PR using the given method ("merge", "squash", or "rebase").
    fn merge_pr(&self, number: u64, merge_method: &str) -> Result<(), GiffError>;
    /// List the repo's open PRs. Used by `giff-runner` to discover stacks via embedded
    /// `giff` metadata blocks; capped at 100 per page (the runner doesn't paginate yet —
    /// 100 open PRs in one repo is the practical scale ceiling for v1).
    fn list_open_pulls(&self) -> Result<Vec<PullRequest>, GiffError>;
}

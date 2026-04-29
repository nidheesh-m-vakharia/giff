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

#[derive(Debug, Clone, Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub html_url: String,
    pub state: String,
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
    /// Merge a PR using the given method ("merge", "squash", or "rebase").
    fn merge_pr(&self, number: u64, merge_method: &str) -> Result<(), GiffError>;
}

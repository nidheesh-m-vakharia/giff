use crate::forge::{CreatePrParams, ForgeBackend, PullRequest, UpdatePrParams};
use giff_core::GiffError;
use serde_json::json;

pub struct GitHubForge {
    token: String,
    repo: String,
    base_url: String,
}

impl GitHubForge {
    pub fn new(token: String, repo: String, base_url: String) -> Self {
        Self {
            token,
            repo,
            base_url,
        }
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/repos/{}{}",
            self.base_url.trim_end_matches('/'),
            self.repo,
            path
        )
    }

    fn agent(&self) -> ureq::Agent {
        ureq::AgentBuilder::new().build()
    }

    fn auth_header(&self) -> String {
        format!("Bearer {}", self.token)
    }
}

impl ForgeBackend for GitHubForge {
    fn create_pr(&self, params: CreatePrParams) -> Result<PullRequest, GiffError> {
        let body = json!({
            "title": params.title,
            "body": params.body,
            "head": params.head,
            "base": params.base,
            "draft": params.draft,
        });
        let resp = self
            .agent()
            .post(&self.url("/pulls"))
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .send_json(body)
            .map_err(|e| GiffError::Forge(e.to_string()))?;
        resp.into_json::<PullRequest>()
            .map_err(|e| GiffError::Forge(e.to_string()))
    }

    fn update_pr(&self, number: u64, params: UpdatePrParams) -> Result<PullRequest, GiffError> {
        let mut body = serde_json::Map::new();
        if let Some(b) = params.body {
            body.insert("body".into(), json!(b));
        }
        if let Some(base) = params.base {
            body.insert("base".into(), json!(base));
        }
        let resp = self
            .agent()
            .patch(&self.url(&format!("/pulls/{}", number)))
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .send_json(serde_json::Value::Object(body))
            .map_err(|e| GiffError::Forge(e.to_string()))?;
        resp.into_json::<PullRequest>()
            .map_err(|e| GiffError::Forge(e.to_string()))
    }

    fn get_pr(&self, number: u64) -> Result<PullRequest, GiffError> {
        let resp = self
            .agent()
            .get(&self.url(&format!("/pulls/{}", number)))
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .call()
            .map_err(|e| GiffError::Forge(e.to_string()))?;
        resp.into_json::<PullRequest>()
            .map_err(|e| GiffError::Forge(e.to_string()))
    }

    fn merge_pr(&self, number: u64, merge_method: &str) -> Result<(), GiffError> {
        let body = json!({ "merge_method": merge_method });
        self.agent()
            .put(&self.url(&format!("/pulls/{}/merge", number)))
            .set("Authorization", &self.auth_header())
            .set("Accept", "application/vnd.github+json")
            .send_json(body)
            .map_err(|e| GiffError::Forge(e.to_string()))?;
        Ok(())
    }
}

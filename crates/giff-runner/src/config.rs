//! TOML config loaded from a path, with `GITHUB_TOKEN` taken from env (so the secret never
//! lives on disk). Per-repo settings cover auto-merge gating and the optional webhook secret
//! for HMAC verification.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_listen")]
    pub listen: SocketAddr,
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,
    /// Polling cadence (seconds). Default is intentionally long — webhooks do the real work
    /// when configured; polling exists as a safety net for missed events.
    #[serde(default = "default_poll_seconds")]
    pub poll_seconds: u64,
    #[serde(default)]
    pub repos: Vec<RepoConfig>,
    /// GitHub Enterprise / on-prem support.
    #[serde(default = "default_github_base_url")]
    pub github_base_url: String,
    /// Token comes from GITHUB_TOKEN env; this is populated after load.
    #[serde(skip)]
    pub github_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    /// `owner/repo`
    pub slug: String,
    /// When true, the worker will merge an open PR sitting at the bottom (root) of any stack
    /// in this repo as soon as it's mergeable AND has at least one approving review.
    #[serde(default)]
    pub auto_merge: bool,
    /// "merge" | "squash" | "rebase". Only consulted when `auto_merge` is true.
    #[serde(default = "default_merge_method")]
    pub merge_method: String,
    /// Shared secret for HMAC-SHA256 webhook signature verification. When unset the runner
    /// rejects all webhook deliveries for this repo (polling-only mode for that repo).
    #[serde(default)]
    pub webhook_secret: Option<String>,
}

fn default_listen() -> SocketAddr {
    "0.0.0.0:8080".parse().unwrap()
}
fn default_data_dir() -> PathBuf {
    PathBuf::from("./data")
}
fn default_poll_seconds() -> u64 {
    900 // 15 minutes
}
fn default_merge_method() -> String {
    "merge".into()
}
fn default_github_base_url() -> String {
    "https://api.github.com".into()
}

impl Config {
    /// Parse a config file. Token is *not* read here — call `with_token` from main, where
    /// the env var lookup happens. Keeps tests from racing on `GITHUB_TOKEN`.
    pub fn load_from(path: &Path) -> Result<Self> {
        let s = std::fs::read_to_string(path)
            .with_context(|| format!("reading config at {}", path.display()))?;
        let cfg: Config = toml::from_str(&s).context("parsing config TOML")?;

        for r in &cfg.repos {
            if !r.slug.contains('/') {
                anyhow::bail!("repo slug `{}` is not in `owner/repo` form", r.slug);
            }
            match r.merge_method.as_str() {
                "merge" | "squash" | "rebase" => {}
                other => anyhow::bail!(
                    "repo `{}`: unknown merge_method `{}` (use merge|squash|rebase)",
                    r.slug,
                    other
                ),
            }
        }

        std::fs::create_dir_all(&cfg.data_dir)
            .with_context(|| format!("creating data dir {}", cfg.data_dir.display()))?;

        Ok(cfg)
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.github_token = token;
        self
    }

    pub fn db_path(&self) -> PathBuf {
        self.data_dir.join("state.db")
    }

    pub fn find_repo(&self, slug: &str) -> Option<&RepoConfig> {
        self.repos.iter().find(|r| r.slug == slug)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parses_minimal_config() {
        let td = TempDir::new().unwrap();
        let cfg_path = td.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            format!(
                r#"
                data_dir = "{}"
                poll_seconds = 60

                [[repos]]
                slug = "owner/repo"
                "#,
                td.path().display()
            ),
        )
        .unwrap();
        let cfg = Config::load_from(&cfg_path).unwrap().with_token("t".into());
        assert_eq!(cfg.repos.len(), 1);
        assert_eq!(cfg.repos[0].slug, "owner/repo");
        assert!(!cfg.repos[0].auto_merge);
        assert_eq!(cfg.repos[0].merge_method, "merge");
        assert_eq!(cfg.poll_seconds, 60);
        assert_eq!(cfg.github_token, "t");
    }

    #[test]
    fn rejects_invalid_slug() {
        let td = TempDir::new().unwrap();
        let cfg_path = td.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            r#"
            [[repos]]
            slug = "not-a-slash"
            "#,
        )
        .unwrap();
        let err = Config::load_from(&cfg_path).unwrap_err();
        assert!(format!("{}", err).contains("owner/repo"));
    }

    #[test]
    fn rejects_unknown_merge_method() {
        let td = TempDir::new().unwrap();
        let cfg_path = td.path().join("config.toml");
        std::fs::write(
            &cfg_path,
            r#"
            [[repos]]
            slug = "o/r"
            merge_method = "bogus"
            "#,
        )
        .unwrap();
        let err = Config::load_from(&cfg_path).unwrap_err();
        assert!(format!("{}", err).contains("merge_method"));
    }
}

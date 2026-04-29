use anyhow::{Context, Result};
use giff_core::StackStore;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct GithubConfig {
    #[serde(default)]
    pub token: String,
    #[serde(default = "default_base_url")]
    pub base_url: String,
}

fn default_base_url() -> String {
    "https://api.github.com".into()
}

fn default_trunk() -> String {
    "main".into()
}

fn default_draft_prs() -> bool {
    true
}

impl Default for GithubConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            base_url: "https://api.github.com".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_trunk")]
    pub trunk: String,
    #[serde(default = "default_draft_prs")]
    pub draft_prs: bool,
    #[serde(default)]
    pub pr_template: String,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            trunk: "main".into(),
            draft_prs: true,
            pr_template: String::new(),
        }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub github: GithubConfig,
    #[serde(default)]
    pub defaults: DefaultsConfig,
}

impl GlobalConfig {
    pub fn load() -> Result<Self> {
        let path = global_config_path()?;
        Self::load_from(path)
    }

    pub fn load_from(path: PathBuf) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let s = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        toml::from_str(&s).with_context(|| "parsing global config")
    }

    pub fn write(&self) -> Result<()> {
        let path = global_config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let s = toml::to_string_pretty(self)?;
        std::fs::write(&path, s)?;
        Ok(())
    }
}

pub fn global_config_path() -> Result<PathBuf> {
    let base = dirs::config_dir().context("could not find config directory")?;
    Ok(base.join("giff").join("config.toml"))
}

/// Read .git/stacked.toml from the given path.
pub fn read_stack_store(path: &Path) -> Result<StackStore> {
    if !path.exists() {
        return Ok(StackStore { stacks: vec![] });
    }
    let s = std::fs::read_to_string(path)?;
    StackStore::from_toml(&s).map_err(|e| anyhow::anyhow!(e))
}

/// Write the stack store to the given path.
pub fn write_stack_store(path: &Path, store: &StackStore) -> Result<()> {
    let s = store.to_toml().map_err(|e| anyhow::anyhow!(e))?;
    std::fs::write(path, s)?;
    Ok(())
}

/// Locate the .git/stacked.toml path by walking up from cwd.
pub fn find_stack_store_path() -> Result<PathBuf> {
    let mut dir = std::env::current_dir()?;
    loop {
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            return Ok(git_dir.join("stacked.toml"));
        }
        if !dir.pop() {
            anyhow::bail!("not inside a git repository");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use giff_core::{FrameId, Stack, StackFrame, StackId};
    use tempfile::TempDir;

    #[test]
    fn global_config_defaults_when_file_missing() {
        let dir = TempDir::new().unwrap();
        let cfg = GlobalConfig::load_from(dir.path().join("config.toml")).unwrap();
        assert_eq!(cfg.github.base_url, "https://api.github.com");
        assert_eq!(cfg.defaults.trunk, "main");
        assert!(cfg.defaults.draft_prs);
    }

    #[test]
    fn stack_store_round_trips_via_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("stacked.toml");
        let store = StackStore {
            stacks: vec![Stack {
                id: StackId("s1".into()),
                name: "test".into(),
                trunk: "main".into(),
                frames: vec![StackFrame {
                    id: FrameId("f1".into()),
                    branch: "feat/a".into(),
                    parent: None,
                    pr_number: None,
                    description: None,
                }],
            }],
        };
        write_stack_store(&path, &store).unwrap();
        let loaded = read_stack_store(&path).unwrap();
        assert_eq!(loaded, store);
    }
}

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum GiffError {
    #[error("no stack found for branch `{0}`")]
    NoStack(String),
    #[error("rebase conflict in frame `{0}` — resolve and run `giff sync --continue`")]
    RebaseConflict(String),
    #[error("GitHub API error: {0}")]
    Forge(String),
    #[error("git error: {0}")]
    Git(String),
    #[error("config error: {0}")]
    Config(String),
    #[error("parse error: {0}")]
    Parse(String),
}

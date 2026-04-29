use giff_core::GiffError;

#[derive(Debug, Clone)]
pub struct Commit {
    pub sha: String,
    pub message: String,
}

#[derive(Debug)]
pub enum RebaseOutcome {
    Clean,
    Conflict { frame: String, hints: Vec<String> },
}

pub trait GitBackend {
    fn current_branch(&self) -> Result<String, GiffError>;
    fn branch_exists(&self, name: &str) -> Result<bool, GiffError>;
    fn commit_log(&self, branch: &str, base: &str) -> Result<Vec<Commit>, GiffError>;
    fn merge_base(&self, a: &str, b: &str) -> Result<String, GiffError>;
    fn create_branch(&self, name: &str, from: &str) -> Result<(), GiffError>;
    fn checkout(&self, branch: &str) -> Result<(), GiffError>;
    fn rebase(&self, branch: &str, onto: &str) -> Result<RebaseOutcome, GiffError>;
    fn push(&self, branch: &str, force: bool) -> Result<(), GiffError>;
}

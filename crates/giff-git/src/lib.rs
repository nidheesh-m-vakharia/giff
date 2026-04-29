pub mod backend;
pub mod shell;

pub use backend::{Commit, GitBackend, RebaseOutcome};
pub use shell::ShellGitBackend;

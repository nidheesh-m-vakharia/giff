use crate::backend::{Commit, GitBackend, RebaseOutcome};
use giff_core::GiffError;
use std::path::PathBuf;
use std::process::Command;

pub struct ShellGitBackend {
    repo_path: PathBuf,
}

impl ShellGitBackend {
    pub fn new(repo_path: PathBuf) -> Self {
        Self { repo_path }
    }

    fn git(&self, args: &[&str]) -> Result<String, GiffError> {
        self.git_raw(args)
    }

    pub fn is_rebase_in_progress(&self) -> bool {
        self.repo_path.join(".git").join("rebase-merge").exists()
            || self.repo_path.join(".git").join("rebase-apply").exists()
    }

    pub fn git_raw(&self, args: &[&str]) -> Result<String, GiffError> {
        let out = Command::new("git")
            .args(args)
            .current_dir(&self.repo_path)
            .output()
            .map_err(|e| GiffError::Git(e.to_string()))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
        } else {
            Err(GiffError::Git(
                String::from_utf8_lossy(&out.stderr).trim().to_string(),
            ))
        }
    }
}

impl GitBackend for ShellGitBackend {
    fn current_branch(&self) -> Result<String, GiffError> {
        self.git(&["rev-parse", "--abbrev-ref", "HEAD"])
    }

    fn branch_exists(&self, name: &str) -> Result<bool, GiffError> {
        Ok(self.git(&["rev-parse", "--verify", name]).is_ok())
    }

    fn commit_log(&self, branch: &str, base: &str) -> Result<Vec<Commit>, GiffError> {
        let range = format!("{}..{}", base, branch);
        let out = self.git(&["log", "--oneline", &range])?;
        let commits = out
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| {
                let (sha, msg) = l.split_once(' ').unwrap_or((l, ""));
                Commit {
                    sha: sha.to_string(),
                    message: msg.to_string(),
                }
            })
            .collect();
        Ok(commits)
    }

    fn merge_base(&self, a: &str, b: &str) -> Result<String, GiffError> {
        self.git(&["merge-base", a, b])
    }

    fn create_branch(&self, name: &str, from: &str) -> Result<(), GiffError> {
        self.git(&["branch", name, from])?;
        Ok(())
    }

    fn checkout(&self, branch: &str) -> Result<(), GiffError> {
        self.git(&["checkout", branch])?;
        Ok(())
    }

    fn rebase(&self, branch: &str, onto: &str) -> Result<RebaseOutcome, GiffError> {
        let result = self.git(&["rebase", onto, branch]);
        match result {
            Ok(_) => Ok(RebaseOutcome::Clean),
            Err(GiffError::Git(msg)) if msg.contains("CONFLICT") || msg.contains("conflict") => {
                // Leave the repo in conflict state so the user can resolve it.
                // The caller saves resume state and tells the user what to do next.
                Ok(RebaseOutcome::Conflict {
                    frame: branch.to_string(),
                    hints: vec![msg],
                })
            }
            Err(e) => Err(e),
        }
    }

    fn push(&self, branch: &str, force: bool) -> Result<(), GiffError> {
        if force {
            self.git(&["push", "--force-with-lease", "origin", branch])?;
        } else {
            self.git(&["push", "origin", branch])?;
        }
        Ok(())
    }
}

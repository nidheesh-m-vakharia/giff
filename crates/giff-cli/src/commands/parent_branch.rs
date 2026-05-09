// Hidden subcommand. Used by the pre-commit hook to look up the current frame's parent
// branch. Always exits 0 so the hook never fails closed: empty output ⇒ "not in a stack,
// skip enforcement".

use crate::config::{find_stack_store_path, read_stack_store};
use anyhow::Result;
use giff_git::{GitBackend, ShellGitBackend};

pub fn run() -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = match backend.current_branch() {
        Ok(b) => b,
        Err(_) => return Ok(()),
    };

    let store_path = match find_stack_store_path() {
        Ok(p) => p,
        Err(_) => return Ok(()),
    };
    let store = match read_stack_store(&store_path) {
        Ok(s) => s,
        Err(_) => return Ok(()),
    };

    let Some((stack, frame)) = store.find_stack_for_branch(&current) else {
        return Ok(());
    };

    let parent_branch = match frame.parent.as_ref() {
        None => stack.trunk.clone(),
        Some(pid) => match stack.frame(pid) {
            Some(f) => f.branch.clone(),
            None => return Ok(()),
        },
    };
    println!("{}", parent_branch);
    Ok(())
}

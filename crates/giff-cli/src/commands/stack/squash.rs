// crates/giff-cli/src/commands/stack/squash.rs
use crate::config::{find_stack_store_path, read_stack_store, write_stack_store};
use anyhow::Result;
use giff_git::{GitBackend, ShellGitBackend};

pub fn run(branch: &str) -> Result<()> {
    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;
    let backend = ShellGitBackend::new(std::env::current_dir()?);

    let stack_id = store
        .find_stack_for_branch(branch)
        .map(|(s, _)| s.id.clone())
        .ok_or_else(|| anyhow::anyhow!("frame `{}` not in a stack", branch))?;

    let stack = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
    let idx = stack
        .frames
        .iter()
        .position(|f| f.branch == branch)
        .unwrap();

    if idx == 0 {
        anyhow::bail!("cannot squash the bottom frame — nothing below it");
    }

    let parent_branch = stack.frames[idx - 1].branch.clone();

    backend.checkout(&parent_branch)?;
    backend.git_raw(&["merge", "--squash", branch])?;
    backend.git_raw(&["commit", "--no-edit"])?;

    stack.frames.remove(idx);
    write_stack_store(&store_path, &store)?;
    println!("Squashed `{}` into `{}`.", branch, parent_branch);
    Ok(())
}

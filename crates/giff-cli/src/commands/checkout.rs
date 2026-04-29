use crate::config::{find_stack_store_path, read_stack_store};
use anyhow::Result;
use giff_git::{GitBackend, ShellGitBackend};

pub fn run(target: &str) -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let current = backend.current_branch()?;

    let branch = if let Ok(pos) = target.parse::<usize>() {
        let (stack, _) = store
            .find_stack_for_branch(&current)
            .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;
        let frames = stack.ordered_frames();
        let idx = pos
            .checked_sub(1)
            .ok_or_else(|| anyhow::anyhow!("position must be >= 1"))?;
        frames
            .get(idx)
            .ok_or_else(|| anyhow::anyhow!("position {} out of range", pos))?
            .branch
            .clone()
    } else {
        target.to_string()
    };

    backend.checkout(&branch)?;
    println!("Checked out: {}", branch);
    Ok(())
}

pub fn run_next() -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let (stack, frame) = store
        .find_stack_for_branch(&current)
        .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;
    let above = stack
        .frame_above(&frame.id)
        .ok_or_else(|| anyhow::anyhow!("already at top of stack"))?;
    backend.checkout(&above.branch)?;
    println!("Checked out: {}", above.branch);
    Ok(())
}

pub fn run_prev() -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let (stack, frame) = store
        .find_stack_for_branch(&current)
        .ok_or_else(|| anyhow::anyhow!("not in a stack"))?;
    let below = stack
        .frame_below(&frame.id)
        .ok_or_else(|| anyhow::anyhow!("already at bottom of stack"))?;
    backend.checkout(&below.branch)?;
    println!("Checked out: {}", below.branch);
    Ok(())
}

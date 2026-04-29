use crate::config::{find_stack_store_path, read_stack_store};
use anyhow::Result;
use giff_git::{GitBackend, ShellGitBackend};

pub fn run() -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;

    println!("branch: {}", current);

    if let Some((stack, frame)) = store.find_stack_for_branch(&current) {
        let pos = stack
            .ordered_frames()
            .iter()
            .position(|f| f.id == frame.id)
            .unwrap_or(0)
            + 1;
        let total = stack.frames.len();
        println!("stack:  {} ({}/{})", stack.name, pos, total);
        if let Some(pr) = frame.pr_number {
            println!("PR:     #{}", pr);
        } else {
            println!("PR:     none (run `giff push` to open)");
        }
    } else {
        println!("not in a stack");
    }
    Ok(())
}

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
        let path: Vec<&str> = stack
            .path_to_root(&frame.id)
            .iter()
            .map(|f| f.branch.as_str())
            .collect();
        let depth = stack.depth(&frame.id);
        let kids = stack.children(&frame.id).len();
        let shape = if stack.is_linear() { "linear" } else { "tree" };

        println!(
            "stack:  {} ({} frames, {})",
            stack.name,
            stack.frames.len(),
            shape
        );
        println!(
            "path:   {} {}{}",
            stack.trunk,
            if path.is_empty() { "" } else { "→ " },
            path.join(" → ")
        );
        println!("depth:  {} (children: {})", depth, kids);
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

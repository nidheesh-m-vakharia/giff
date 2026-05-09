// crates/giff-cli/src/commands/stack/drop.rs
use crate::config::{find_stack_store_path, read_stack_store, write_stack_store};
use anyhow::Result;

pub fn run(branch: &str) -> Result<()> {
    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;

    let stack_id = store
        .find_stack_for_branch(branch)
        .map(|(s, _)| s.id.clone())
        .ok_or_else(|| anyhow::anyhow!("frame `{}` not found in any stack", branch))?;

    let stack = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
    let drop_idx = stack
        .frames
        .iter()
        .position(|f| f.branch == branch)
        .ok_or_else(|| anyhow::anyhow!("frame not found"))?;

    let dropped = stack.frames.remove(drop_idx);

    // Re-parent every direct child of the dropped frame to the dropped frame's parent.
    // For a linear stack this affects exactly one frame; for trees, every sibling-child gets
    // promoted up one level under their grandparent (or made into roots if dropped was a root).
    for f in stack.frames.iter_mut() {
        if f.parent.as_ref() == Some(&dropped.id) {
            f.parent = dropped.parent.clone();
        }
    }

    stack.validate()?;
    write_stack_store(&store_path, &store)?;
    println!("Dropped frame: {}", branch);
    println!(
        "Note: branch `{}` still exists locally. Delete it with `git branch -D {}`.",
        branch, branch
    );
    Ok(())
}

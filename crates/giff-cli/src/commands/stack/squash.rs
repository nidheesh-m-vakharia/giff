// crates/giff-cli/src/commands/stack/squash.rs
use crate::config::{find_stack_store_path, read_stack_store, write_stack_store};
use anyhow::Result;
use giff_core::{FrameId, Stack};
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

    let target_id = stack.frames[idx]
        .parent
        .clone()
        .ok_or_else(|| anyhow::anyhow!("cannot squash a root frame — it has no parent below it"))?;

    let parent_branch = stack
        .frames
        .iter()
        .find(|f| f.id == target_id)
        .map(|f| f.branch.clone())
        .ok_or_else(|| anyhow::anyhow!("parent frame missing"))?;

    // To preserve "one commit per frame", we amend the parent's existing commit with the
    // squashed-in changes rather than adding a new commit on top of it. Refuse if the parent
    // has no commits to amend, or if the child has nothing to squash.
    let grandparent_branch = grandparent_branch_for(stack, &target_id);
    let parent_count = commits_ahead(&backend, &parent_branch, &grandparent_branch)?;
    if parent_count == 0 {
        anyhow::bail!(
            "parent frame `{}` has no commits to absorb a squash. Commit on the parent first, \
             or drop this frame instead with `giff stack drop {}`.",
            parent_branch, branch
        );
    }
    let child_count = commits_ahead(&backend, branch, &parent_branch)?;
    if child_count == 0 {
        anyhow::bail!(
            "frame `{}` has no commits to squash into `{}`.",
            branch,
            parent_branch
        );
    }

    backend.checkout(&parent_branch)?;
    backend.git_raw(&["merge", "--squash", branch])?;
    backend.git_raw(&["commit", "--amend", "--no-edit"])?;

    let squashed_id = stack.frames[idx].id.clone();

    // Children of the squashed frame get re-parented to the squash target. In a linear stack
    // there's at most one child; in a tree, all children move up one level together.
    for f in stack.frames.iter_mut() {
        if f.parent.as_ref() == Some(&squashed_id) {
            f.parent = Some(target_id.clone());
        }
    }

    stack.frames.remove(idx);
    stack.validate()?;
    write_stack_store(&store_path, &store)?;
    println!("Squashed `{}` into `{}`.", branch, parent_branch);
    Ok(())
}

/// Branch name to use as `git rev-list <base>..<branch>` for counting `target`'s own commits.
/// `target` is the parent frame the squash will land on; its base is *its* parent (or trunk).
fn grandparent_branch_for(stack: &Stack, target_id: &FrameId) -> String {
    let target = stack.frame(target_id).unwrap();
    match target.parent.as_ref() {
        None => stack.trunk.clone(),
        Some(gid) => stack
            .frame(gid)
            .map(|f| f.branch.clone())
            .unwrap_or_else(|| stack.trunk.clone()),
    }
}

fn commits_ahead(backend: &ShellGitBackend, branch: &str, base: &str) -> Result<usize> {
    let out = backend.git_raw(&["rev-list", "--count", &format!("{}..{}", base, branch)])?;
    Ok(out.trim().parse::<usize>().unwrap_or(0))
}

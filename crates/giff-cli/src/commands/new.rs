use crate::config::{find_stack_store_path, read_stack_store, write_stack_store, GlobalConfig};
use anyhow::Result;
use giff_core::{FrameId, Stack, StackFrame, StackId};
use giff_git::{GitBackend, ShellGitBackend};
use uuid::Uuid;

pub fn run(branch: &str) -> Result<()> {
    let cfg = GlobalConfig::load()?;
    let repo_path = std::env::current_dir()?;
    let backend = ShellGitBackend::new(repo_path);
    let current = backend.current_branch()?;

    let store_path = find_stack_store_path()?;
    let mut store = read_stack_store(&store_path)?;

    // Find which stack the current branch belongs to, if any.
    let parent_frame_id = store
        .find_stack_for_branch(&current)
        .map(|(_, f)| f.id.clone());

    if let Some((stack, _)) = store.find_stack_for_branch(&current) {
        // Adding on top of an existing stack frame.
        let stack_id = stack.id.clone();
        let new_frame = StackFrame {
            id: FrameId(Uuid::new_v4().to_string()),
            branch: branch.to_string(),
            parent: parent_frame_id,
            pr_number: None,
            description: None,
        };
        let s = store.stacks.iter_mut().find(|s| s.id == stack_id).unwrap();
        s.frames.push(new_frame);
    } else if current == cfg.defaults.trunk {
        // Starting a new stack from trunk — trunk is not a frame.
        let frame = StackFrame {
            id: FrameId(Uuid::new_v4().to_string()),
            branch: branch.to_string(),
            parent: None,
            pr_number: None,
            description: None,
        };
        store.stacks.push(Stack {
            id: StackId(Uuid::new_v4().to_string()),
            name: branch.to_string(),
            trunk: current.clone(),
            frames: vec![frame],
        });
    } else {
        // Starting a new stack from a non-trunk branch — current branch becomes the bottom frame.
        let bottom = StackFrame {
            id: FrameId(Uuid::new_v4().to_string()),
            branch: current.clone(),
            parent: None,
            pr_number: None,
            description: None,
        };
        let top = StackFrame {
            id: FrameId(Uuid::new_v4().to_string()),
            branch: branch.to_string(),
            parent: Some(bottom.id.clone()),
            pr_number: None,
            description: None,
        };
        store.stacks.push(Stack {
            id: StackId(Uuid::new_v4().to_string()),
            name: branch.to_string(),
            trunk: cfg.defaults.trunk.clone(),
            frames: vec![bottom, top],
        });
    }

    backend.create_branch(branch, &current)?;
    backend.checkout(branch)?;
    write_stack_store(&store_path, &store)?;

    println!("Created frame: {branch}");
    Ok(())
}

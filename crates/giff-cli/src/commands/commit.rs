// crates/giff-cli/src/commands/commit.rs
//
// `giff commit` enforces the "one commit per frame" invariant. The first commit on a frame is
// allowed; a second is refused with instructions to either start a new frame (`giff new`) or
// amend the existing one (`giff commit --amend`). `--amend` is always allowed because it
// preserves the invariant.

use crate::config::{find_stack_store_path, read_stack_store};
use anyhow::Result;
use giff_git::{GitBackend, ShellGitBackend};

pub fn run(message: Option<&str>, amend: bool, all: bool) -> Result<()> {
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let current = backend.current_branch()?;

    let (stack, frame) = store.find_stack_for_branch(&current).ok_or_else(|| {
        anyhow::anyhow!(
            "branch `{}` is not in a stack — run `giff new <branch>` first",
            current
        )
    })?;

    let parent_branch = match frame.parent.as_ref() {
        None => stack.trunk.clone(),
        Some(pid) => stack
            .frame(pid)
            .ok_or_else(|| anyhow::anyhow!("parent frame missing for `{}`", current))?
            .branch
            .clone(),
    };

    let count = commits_ahead(&backend, &parent_branch)?;

    if !amend && count >= 1 {
        anyhow::bail!(
            "frame `{}` already has {} commit(s) ahead of `{}` — one commit per frame is enforced.\n\
             Options:\n  \
               • Start a new frame on top:   giff new <branch-name>\n  \
               • Amend the existing commit:  giff commit --amend [-m \"<message>\"]",
            current,
            count,
            parent_branch
        );
    }

    if !amend && message.is_none() {
        anyhow::bail!("commit message required: pass `-m \"<message>\"`");
    }

    let mut args: Vec<String> = vec!["commit".into()];
    if amend {
        args.push("--amend".into());
        if message.is_none() {
            // Keep the existing message; otherwise git would open the editor.
            args.push("--no-edit".into());
        }
    }
    if all {
        args.push("-a".into());
    }
    if let Some(m) = message {
        args.push("-m".into());
        args.push(m.into());
    }

    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let output = backend.git_raw(&arg_refs)?;
    if !output.is_empty() {
        println!("{}", output);
    }
    println!(
        "{} on `{}` (1 commit ahead of `{}`).",
        if amend { "Amended" } else { "Committed" },
        current,
        parent_branch
    );
    Ok(())
}

fn commits_ahead(backend: &ShellGitBackend, parent_branch: &str) -> Result<usize> {
    let out = backend.git_raw(&["rev-list", "--count", &format!("{}..HEAD", parent_branch)])?;
    Ok(out.trim().parse::<usize>().unwrap_or(0))
}

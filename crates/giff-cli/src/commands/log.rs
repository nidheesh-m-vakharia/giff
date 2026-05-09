use crate::commands::push::parse_github_repo;
use crate::config::{find_stack_store_path, read_stack_store, GlobalConfig};
use anyhow::Result;
use giff_core::{FrameId, Stack, StackFrame};
use giff_git::{GitBackend, ShellGitBackend};
use giff_github::{ForgeBackend, GitHubForge};
use std::collections::HashMap;

pub fn run(show_all: bool) -> Result<()> {
    let store_path = find_stack_store_path()?;
    let store = read_stack_store(&store_path)?;
    let backend = ShellGitBackend::new(std::env::current_dir()?);
    let current = backend.current_branch()?;

    if store.stacks.is_empty() {
        println!("No stacks. Run `giff new <branch>` to create one.");
        return Ok(());
    }

    let forge = build_forge(&backend);
    let pr_states = match forge.as_ref() {
        Some(f) => fetch_pr_states_parallel(&store.stacks, f),
        None => HashMap::new(),
    };

    let mut hidden = 0usize;
    let is_visible = |pr_number: Option<u64>| -> bool {
        if show_all {
            return true;
        }
        match pr_number {
            None => true, // unpushed frames are always shown
            Some(n) => pr_states
                .get(&n)
                .map(|s| s.as_str() != "closed")
                .unwrap_or(true),
        }
    };

    for stack in &store.stacks {
        let any_visible = stack.frames.iter().any(|f| is_visible(f.pr_number));
        if !any_visible {
            hidden += stack.frames.len();
            continue;
        }

        println!("stack: {} (trunk: {})", stack.name, stack.trunk);
        println!("● {}", stack.trunk);

        // Hidden roots are absorbed — their visible descendants become the visible "roots."
        // This keeps the rendered tree contiguous after frames merge.
        let visible_roots = visible_children_of_root(stack, &is_visible, &mut hidden);
        for root in visible_roots {
            println!("│");
            print_frame(root, "", &current, &pr_states);
            print_visible_subtree(stack, &root.id, "", &current, &pr_states, &is_visible, &mut hidden);
        }
        println!();
    }

    if hidden > 0 && !show_all {
        println!(
            "({} frame{} hidden — pass `--all` to include closed/merged PRs)",
            hidden,
            if hidden == 1 { "" } else { "s" }
        );
    }
    Ok(())
}

/// First layer of visible frames in the stack. `roots()` are the natural starting point;
/// hidden roots get unwound to expose their visible descendants.
fn visible_children_of_root<'a>(
    stack: &'a Stack,
    is_visible: &impl Fn(Option<u64>) -> bool,
    hidden: &mut usize,
) -> Vec<&'a StackFrame> {
    let mut out = Vec::new();
    for root in stack.roots() {
        if is_visible(root.pr_number) {
            out.push(root);
        } else {
            *hidden += 1;
            out.extend(visible_descendants_one_level(stack, &root.id, is_visible, hidden));
        }
    }
    out
}

/// Walk `parent_id`'s direct children. Visible ones come back as-is; hidden ones get
/// unwound recursively to expose THEIR visible descendants. The `hidden` counter tracks
/// the unwound nodes so the user can see the trim count.
fn visible_descendants_one_level<'a>(
    stack: &'a Stack,
    parent_id: &FrameId,
    is_visible: &impl Fn(Option<u64>) -> bool,
    hidden: &mut usize,
) -> Vec<&'a StackFrame> {
    let mut out = Vec::new();
    for kid in stack.children(parent_id) {
        if is_visible(kid.pr_number) {
            out.push(kid);
        } else {
            *hidden += 1;
            out.extend(visible_descendants_one_level(stack, &kid.id, is_visible, hidden));
        }
    }
    out
}

fn print_visible_subtree(
    stack: &Stack,
    parent_id: &FrameId,
    prefix: &str,
    current: &str,
    pr_states: &HashMap<u64, String>,
    is_visible: &impl Fn(Option<u64>) -> bool,
    hidden: &mut usize,
) {
    let kids = visible_descendants_one_level(stack, parent_id, is_visible, hidden);
    let n = kids.len();
    for (i, kid) in kids.iter().enumerate() {
        let is_last = i + 1 == n;
        let connector = if is_last { "└─ " } else { "├─ " };
        let descent = if is_last { "   " } else { "│  " };
        println!("{}│", prefix);
        print_frame(kid, &format!("{}{}", prefix, connector), current, pr_states);
        print_visible_subtree(
            stack,
            &kid.id,
            &format!("{}{}", prefix, descent),
            current,
            pr_states,
            is_visible,
            hidden,
        );
    }
}

fn fetch_pr_states_parallel(stacks: &[Stack], forge: &GitHubForge) -> HashMap<u64, String> {
    let pr_numbers: Vec<u64> = stacks
        .iter()
        .flat_map(|s| s.frames.iter().filter_map(|f| f.pr_number))
        .collect();
    let results = crate::concurrency::parallel_map(
        pr_numbers,
        crate::concurrency::HTTP_WORKERS,
        |&pr_num| (pr_num, forge.get_pr(pr_num).ok().map(|pr| (pr.state, pr.merged))),
    );
    results
        .into_iter()
        .filter_map(|(pr_num, info)| {
            info.map(|(state, merged)| {
                // Distinguish merged from rejected — both are `state == "closed"` on GitHub,
                // but for the user "merged" is good news. Render as such.
                let label = if merged { "merged".to_string() } else { state };
                (pr_num, label)
            })
        })
        .collect()
}

fn print_frame(
    frame: &StackFrame,
    prefix: &str,
    current: &str,
    pr_states: &HashMap<u64, String>,
) {
    let marker = if frame.branch == current {
        " ← you are here"
    } else {
        ""
    };
    let pr_label = match frame.pr_number {
        None => "no PR".to_string(),
        Some(n) => {
            let state = pr_states.get(&n).cloned().unwrap_or_else(|| format!("#{}", n));
            format!("PR #{} [{}]", n, state)
        }
    };
    println!("{}◉ {}  [{}]{}", prefix, frame.branch, pr_label, marker);
}

fn build_forge(backend: &ShellGitBackend) -> Option<GitHubForge> {
    let cfg = GlobalConfig::load().ok()?;
    let token = std::env::var("GITHUB_TOKEN")
        .ok()
        .filter(|t| !t.is_empty())
        .or_else(|| {
            if cfg.github.token.is_empty() {
                None
            } else {
                Some(cfg.github.token.clone())
            }
        })?;
    let remote_url = backend.git_raw(&["remote", "get-url", "origin"]).ok()?;
    let repo = parse_github_repo(&remote_url)?;
    Some(GitHubForge::new(token, repo, cfg.github.base_url))
}

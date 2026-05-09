//! Reconstruct stacks from a list of PR snapshots by parsing each PR's embedded `giff`
//! metadata block. Mirrors the logic in `apps/web/src/lib/api/stack.ts`. Pure function: no
//! I/O, no side effects, fully testable.
//!
//! The runner doesn't persist computed stack structure — it stores raw PR snapshots and
//! rebuilds stacks on demand. Cheaper than keeping derived state in sync.

use crate::db::PullSnapshot;
use giff_core::{FrameId, RemoteStackMeta, StackId};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct GroupedStacks {
    pub stacks: Vec<Stack>,
    pub ungrouped: Vec<PullSnapshot>,
}

#[derive(Debug, Serialize)]
pub struct Stack {
    pub id: String,
    pub total: usize,
    pub is_linear: bool,
    /// Topologically ordered (parents before children).
    pub frames: Vec<Frame>,
    /// Tree shape; one or more roots when the stack contains parallel children.
    pub roots: Vec<TreeNode>,
}

#[derive(Debug, Serialize)]
pub struct Frame {
    pub pr: PullSnapshot,
    pub meta: RemoteStackMeta,
}

#[derive(Debug, Serialize)]
pub struct TreeNode {
    pub pr_number: u64,
    pub frame_id: String,
    pub head_ref: String,
    pub children: Vec<TreeNode>,
}

/// Group `pulls` (typically all PRs across a repo or all repos) into stacks. Open and
/// merged PRs both contribute — having recently-merged PRs in the structure is useful for
/// the `events` view and any retroactive reconciliation.
pub fn group(pulls: Vec<PullSnapshot>) -> GroupedStacks {
    let mut buckets: HashMap<String, Vec<Frame>> = HashMap::new();
    let mut ungrouped: Vec<PullSnapshot> = Vec::new();

    for pr in pulls {
        match pr.body.as_deref().and_then(RemoteStackMeta::from_pr_body) {
            Some(meta) => {
                buckets
                    .entry(meta.stack_id.0.clone())
                    .or_default()
                    .push(Frame { pr, meta });
            }
            None => ungrouped.push(pr),
        }
    }

    let mut stacks: Vec<Stack> = buckets
        .into_iter()
        .map(|(id, frames)| build_stack(StackId(id), frames))
        .collect();

    // Most-recently-touched stacks first (by max updated_at across frames).
    stacks.sort_by(|a, b| {
        let max_a = a
            .frames
            .iter()
            .map(|f| f.pr.updated_at.as_str())
            .max()
            .unwrap_or("");
        let max_b = b
            .frames
            .iter()
            .map(|f| f.pr.updated_at.as_str())
            .max()
            .unwrap_or("");
        max_b.cmp(max_a)
    });

    GroupedStacks { stacks, ungrouped }
}

fn build_stack(id: StackId, mut frames: Vec<Frame>) -> Stack {
    // Index by frame_id once.
    let by_id: HashMap<FrameId, &Frame> = frames
        .iter()
        .map(|f| (f.meta.frame_id.clone(), f))
        .collect();

    // Children buckets, keyed by parent frame_id. None goes to "roots".
    let mut children_of: HashMap<FrameId, Vec<FrameId>> = HashMap::new();
    let mut root_ids: Vec<FrameId> = Vec::new();
    for f in &frames {
        match f.meta.parent_frame_id.as_ref() {
            Some(parent) if by_id.contains_key(parent) => {
                children_of
                    .entry(parent.clone())
                    .or_default()
                    .push(f.meta.frame_id.clone());
            }
            _ => root_ids.push(f.meta.frame_id.clone()),
        }
    }

    // Stable ordering: sort siblings by `position` so renders are deterministic.
    let sort_by_pos = |ids: &mut Vec<FrameId>, by_id: &HashMap<FrameId, &Frame>| {
        ids.sort_by_key(|id| by_id.get(id).map(|f| f.meta.position).unwrap_or(usize::MAX));
    };
    sort_by_pos(&mut root_ids, &by_id);
    for kids in children_of.values_mut() {
        sort_by_pos(kids, &by_id);
    }

    // Build tree nodes recursively + collect frames in topological order.
    let mut topo: Vec<FrameId> = Vec::with_capacity(frames.len());
    fn walk(
        id: &FrameId,
        by_id: &HashMap<FrameId, &Frame>,
        children_of: &HashMap<FrameId, Vec<FrameId>>,
        topo: &mut Vec<FrameId>,
    ) -> TreeNode {
        topo.push(id.clone());
        let frame = by_id.get(id).unwrap();
        let kids = children_of
            .get(id)
            .into_iter()
            .flatten()
            .map(|c| walk(c, by_id, children_of, topo))
            .collect();
        TreeNode {
            pr_number: frame.pr.number,
            frame_id: id.0.clone(),
            head_ref: frame.pr.head_ref.clone(),
            children: kids,
        }
    }
    let roots: Vec<TreeNode> = root_ids
        .iter()
        .map(|id| walk(id, &by_id, &children_of, &mut topo))
        .collect();

    let is_linear = roots.len() == 1 && children_of.values().all(|kids| kids.len() <= 1);

    // Re-order `frames` to match topological order.
    let pos: HashMap<&FrameId, usize> = topo.iter().enumerate().map(|(i, id)| (id, i)).collect();
    frames.sort_by_key(|f| pos.get(&f.meta.frame_id).copied().unwrap_or(usize::MAX));

    let total = frames.first().map(|f| f.meta.total).unwrap_or(frames.len());

    Stack {
        id: id.0,
        total,
        is_linear,
        frames,
        roots,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(repo: &str, num: u64, head: &str, body: &str) -> PullSnapshot {
        PullSnapshot {
            repo: repo.into(),
            number: num,
            title: format!("PR #{}", num),
            state: "open".into(),
            merged: false,
            draft: false,
            head_ref: head.into(),
            base_ref: "main".into(),
            body: Some(body.into()),
            html_url: format!("https://example.com/{}", num),
            updated_at: format!("2026-05-0{}T00:00:00Z", num),
            seen_at: num as i64,
        }
    }

    fn block(
        stack: &str,
        frame: &str,
        parent: Option<&str>,
        position: usize,
        total: usize,
    ) -> String {
        let parent = parent
            .map(|p| format!(r#""parent_frame_id":"{}","#, p))
            .unwrap_or_default();
        format!(
            "Part {pos}/{total} of stack `{s}`.\n\n```giff\n{{\"stack_id\":\"{s}\",\"frame_id\":\"{f}\",{p}\"position\":{pos},\"total\":{total}}}\n```",
            pos = position,
            total = total,
            s = stack,
            f = frame,
            p = parent
        )
    }

    #[test]
    fn groups_linear_stack() {
        let pulls = vec![
            snapshot("o/r", 1, "feat/a", &block("s1", "f1", None, 1, 3)),
            snapshot("o/r", 2, "feat/b", &block("s1", "f2", Some("f1"), 2, 3)),
            snapshot("o/r", 3, "feat/c", &block("s1", "f3", Some("f2"), 3, 3)),
        ];
        let g = group(pulls);
        assert_eq!(g.stacks.len(), 1);
        assert_eq!(g.ungrouped.len(), 0);
        let stack = &g.stacks[0];
        assert!(stack.is_linear);
        assert_eq!(stack.frames.len(), 3);
        // Topo order: f1 → f2 → f3
        assert_eq!(stack.frames[0].meta.frame_id.0, "f1");
        assert_eq!(stack.frames[1].meta.frame_id.0, "f2");
        assert_eq!(stack.frames[2].meta.frame_id.0, "f3");
    }

    #[test]
    fn groups_y_shape() {
        let pulls = vec![
            snapshot("o/r", 1, "feat/root", &block("y", "f1", None, 1, 3)),
            snapshot("o/r", 2, "feat/left", &block("y", "f2", Some("f1"), 2, 3)),
            snapshot("o/r", 3, "feat/right", &block("y", "f3", Some("f1"), 3, 3)),
        ];
        let g = group(pulls);
        assert_eq!(g.stacks.len(), 1);
        let s = &g.stacks[0];
        assert!(!s.is_linear);
        assert_eq!(s.roots.len(), 1);
        assert_eq!(s.roots[0].children.len(), 2);
    }

    #[test]
    fn ungrouped_prs_have_no_block() {
        let pulls = vec![
            snapshot("o/r", 1, "feat/a", &block("s1", "f1", None, 1, 1)),
            snapshot("o/r", 99, "fix/typo", "no block here"),
        ];
        let g = group(pulls);
        assert_eq!(g.stacks.len(), 1);
        assert_eq!(g.ungrouped.len(), 1);
        assert_eq!(g.ungrouped[0].number, 99);
    }
}

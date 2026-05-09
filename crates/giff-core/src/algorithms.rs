use crate::{FrameId, GiffError, Stack, StackFrame, StackStore};
use std::collections::{HashMap, HashSet};

impl StackStore {
    /// Find the stack and frame for a given branch name.
    pub fn find_stack_for_branch(&self, branch: &str) -> Option<(&Stack, &StackFrame)> {
        for stack in &self.stacks {
            if let Some(frame) = stack.frames.iter().find(|f| f.branch == branch) {
                return Some((stack, frame));
            }
        }
        None
    }
}

impl Stack {
    /// All frames whose `parent` is `None` — i.e. frames that target the trunk directly.
    /// A well-formed linear stack has exactly one root; trees with multiple roots are valid
    /// (e.g. "two parallel branches off main"), but `giff stack land` requires exactly one.
    pub fn roots(&self) -> Vec<&StackFrame> {
        self.frames.iter().filter(|f| f.parent.is_none()).collect()
    }

    /// Direct children of `id` — frames whose `parent` points at it.
    /// Returns `Vec` because trees allow multiple children. For linear stacks this is 0 or 1.
    pub fn children(&self, id: &FrameId) -> Vec<&StackFrame> {
        self.frames
            .iter()
            .filter(|f| f.parent.as_ref() == Some(id))
            .collect()
    }

    /// Look up a frame by id.
    pub fn frame(&self, id: &FrameId) -> Option<&StackFrame> {
        self.frames.iter().find(|f| &f.id == id)
    }

    /// The frame directly below `id` (its parent), or `None` if `id` is a root.
    pub fn frame_below(&self, id: &FrameId) -> Option<&StackFrame> {
        let frame = self.frame(id)?;
        let parent_id = frame.parent.as_ref()?;
        self.frame(parent_id)
    }

    /// Recursive descendants of `id` in pre-order (the subtree rooted at `id`,
    /// excluding `id` itself).
    pub fn descendants(&self, id: &FrameId) -> Vec<&StackFrame> {
        let mut out = Vec::new();
        let mut stack = vec![id.clone()];
        while let Some(cur) = stack.pop() {
            for child in self.children(&cur) {
                out.push(child);
                stack.push(child.id.clone());
            }
        }
        out
    }

    /// Chain of ancestors from the parent of `id` up to a root.
    /// First element is `id`'s parent, last element is the root.
    pub fn ancestors(&self, id: &FrameId) -> Vec<&StackFrame> {
        let mut out = Vec::new();
        let mut cursor = self.frame(id).and_then(|f| f.parent.as_ref()).cloned();
        while let Some(cur) = cursor {
            if let Some(frame) = self.frame(&cur) {
                out.push(frame);
                cursor = frame.parent.clone();
            } else {
                break;
            }
        }
        out
    }

    /// Path from a root down to `id` (inclusive). The first element is the root, the last is `id`.
    pub fn path_to_root(&self, id: &FrameId) -> Vec<&StackFrame> {
        let mut chain: Vec<&StackFrame> = self.ancestors(id);
        chain.reverse();
        if let Some(self_frame) = self.frame(id) {
            chain.push(self_frame);
        }
        chain
    }

    /// Distance from `id` to its root (root has depth 0).
    pub fn depth(&self, id: &FrameId) -> usize {
        self.ancestors(id).len()
    }

    /// All frames in topological order — every parent appears before each of its children.
    /// Stable for a given input (children are visited in `frames` insertion order so renders are
    /// deterministic across runs).
    pub fn topological_order(&self) -> Vec<&StackFrame> {
        let mut out: Vec<&StackFrame> = Vec::with_capacity(self.frames.len());
        let mut visited: HashSet<FrameId> = HashSet::new();
        for root in self.roots() {
            self.dfs_collect(&root.id, &mut visited, &mut out);
        }
        // Defensive: if validate() has been skipped and the stack contains a cycle, some frames
        // may not be reachable from any root. Append them in insertion order so callers still see
        // every frame and can detect the inconsistency (validate() catches the actual cycle).
        for f in &self.frames {
            if !visited.contains(&f.id) {
                out.push(f);
                visited.insert(f.id.clone());
            }
        }
        out
    }

    fn dfs_collect<'a>(
        &'a self,
        id: &FrameId,
        visited: &mut HashSet<FrameId>,
        out: &mut Vec<&'a StackFrame>,
    ) {
        if !visited.insert(id.clone()) {
            return;
        }
        if let Some(frame) = self.frame(id) {
            out.push(frame);
        }
        for child in self.children(id) {
            self.dfs_collect(&child.id, visited, out);
        }
    }

    /// Topological order — was previously the linear bottom-to-top traversal. Now equivalent
    /// to `topological_order()`. Kept for call sites still using this name.
    pub fn ordered_frames(&self) -> Vec<&StackFrame> {
        self.topological_order()
    }

    /// True when the stack is a single linear chain (one root, every frame has at most one child).
    pub fn is_linear(&self) -> bool {
        if self.roots().len() != 1 {
            return false;
        }
        self.frames.iter().all(|f| self.children(&f.id).len() <= 1)
    }

    /// Compute the new parent for every non-merged frame after a set of frames are pruned.
    ///
    /// "Merged" here means "this PR was merged on GitHub, so we should drop the frame from the
    /// local stack and re-parent its descendants up the chain." We walk each non-merged frame's
    /// parent chain, skipping merged ancestors, until we hit a non-merged ancestor or `None`.
    ///
    /// The returned map only contains frames whose parent actually *changed*, so the caller can
    /// push exactly that many `update_pr` calls to the forge and not waste API rate.
    pub fn parent_updates_after_pruning(
        &self,
        merged: &HashSet<FrameId>,
    ) -> HashMap<FrameId, Option<FrameId>> {
        let by_id: HashMap<&FrameId, &StackFrame> =
            self.frames.iter().map(|f| (&f.id, f)).collect();
        let mut out = HashMap::new();
        for frame in &self.frames {
            if merged.contains(&frame.id) {
                continue;
            }
            let mut p = frame.parent.clone();
            while let Some(ref pid) = p {
                if merged.contains(pid) {
                    p = by_id.get(pid).and_then(|f| f.parent.clone());
                } else {
                    break;
                }
            }
            if p != frame.parent {
                out.insert(frame.id.clone(), p);
            }
        }
        out
    }

    /// Validate the stack's tree shape. Returns Err on cycles, missing parents, duplicate ids,
    /// or duplicate branches. Run after every mutation in `giff-cli` as a safety net.
    pub fn validate(&self) -> Result<(), GiffError> {
        // 1. No duplicate frame ids.
        let mut ids: HashMap<&FrameId, ()> = HashMap::new();
        for f in &self.frames {
            if ids.insert(&f.id, ()).is_some() {
                return Err(GiffError::InvalidStack(format!(
                    "duplicate frame id `{}`",
                    f.id.0
                )));
            }
        }

        // 2. No duplicate branch names.
        let mut branches: HashMap<&str, ()> = HashMap::new();
        for f in &self.frames {
            if branches.insert(f.branch.as_str(), ()).is_some() {
                return Err(GiffError::InvalidStack(format!(
                    "duplicate branch `{}` in stack",
                    f.branch
                )));
            }
        }

        // 3. Every parent_id must reference a frame in this stack.
        for f in &self.frames {
            if let Some(p) = f.parent.as_ref() {
                if !ids.contains_key(p) {
                    return Err(GiffError::InvalidStack(format!(
                        "frame `{}` has dangling parent `{}`",
                        f.branch, p.0
                    )));
                }
            }
        }

        // 4. No cycles. Walk every frame upward; if we revisit, we have a cycle.
        for f in &self.frames {
            let mut seen: HashSet<&FrameId> = HashSet::new();
            let mut cur = Some(&f.id);
            while let Some(id) = cur {
                if !seen.insert(id) {
                    return Err(GiffError::InvalidStack(format!(
                        "cycle detected at frame `{}`",
                        f.branch
                    )));
                }
                cur = self
                    .frames
                    .iter()
                    .find(|x| &x.id == id)
                    .and_then(|x| x.parent.as_ref());
            }
        }

        Ok(())
    }
}

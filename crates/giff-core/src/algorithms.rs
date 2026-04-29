use crate::{FrameId, Stack, StackFrame, StackStore};

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
    /// Return frames ordered bottom → top (parent before child).
    /// Assumes frames are already stored bottom → top per the data model invariant.
    pub fn ordered_frames(&self) -> Vec<&StackFrame> {
        self.frames.iter().collect()
    }

    /// Return the frame directly below `id` (the parent frame), or None if `id` is the bottom.
    pub fn frame_below(&self, id: &FrameId) -> Option<&StackFrame> {
        let frame = self.frames.iter().find(|f| &f.id == id)?;
        let parent_id = frame.parent.as_ref()?;
        self.frames.iter().find(|f| &f.id == parent_id)
    }

    /// Return the frame directly above `id` (the child frame), or None if `id` is the top.
    pub fn frame_above(&self, id: &FrameId) -> Option<&StackFrame> {
        self.frames.iter().find(|f| f.parent.as_ref() == Some(id))
    }
}

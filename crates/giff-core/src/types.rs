use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FrameId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StackId(pub String);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackFrame {
    pub id: FrameId,
    pub branch: String,
    /// None for the bottom frame (targets trunk directly).
    pub parent: Option<FrameId>,
    pub pr_number: Option<u64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Stack {
    pub id: StackId,
    pub name: String,
    pub trunk: String,
    /// Ordered bottom → top.
    pub frames: Vec<StackFrame>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StackStore {
    pub stacks: Vec<Stack>,
}

/// Embedded in each PR description as a fenced JSON block.
/// Fence marker: ```giff ... ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RemoteStackMeta {
    pub stack_id: StackId,
    pub frame_id: FrameId,
    pub position: usize,
    pub total: usize,
}

impl RemoteStackMeta {
    /// Produce the fenced block to embed in a PR description.
    pub fn to_pr_block(&self) -> String {
        let json = serde_json::to_string(self).expect("RemoteStackMeta is always serializable");
        format!("```giff\n{}\n```", json)
    }

    /// Parse RemoteStackMeta from a PR description body.
    pub fn from_pr_body(body: &str) -> Option<Self> {
        let start = body.find("```giff\n")?;
        let inner_start = start + "```giff\n".len();
        let rest = &body[inner_start..];
        // Find closing fence: must be "\n```" followed by newline, end-of-string, or non-backtick
        let end = rest.find("\n```").filter(|&pos| {
            let after = &rest[pos + 4..];
            after.is_empty() || after.starts_with('\n') || after.starts_with('\r')
        })?;
        let json = &rest[..end];
        serde_json::from_str(json).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stack_frame_bottom_has_no_parent() {
        let frame = StackFrame {
            id: FrameId("f1".into()),
            branch: "feat/base".into(),
            parent: None,
            pr_number: None,
            description: None,
        };
        assert!(frame.parent.is_none());
    }

    #[test]
    fn stack_frame_child_has_parent() {
        let frame = StackFrame {
            id: FrameId("f2".into()),
            branch: "feat/child".into(),
            parent: Some(FrameId("f1".into())),
            pr_number: Some(42),
            description: Some("adds tokens".into()),
        };
        assert_eq!(frame.parent, Some(FrameId("f1".into())));
        assert_eq!(frame.pr_number, Some(42));
    }

    #[test]
    fn remote_meta_round_trips_through_pr_body() {
        let meta = RemoteStackMeta {
            stack_id: StackId("s1".into()),
            frame_id: FrameId("f2".into()),
            position: 2,
            total: 4,
        };
        let body = format!(
            "This PR adds tokens.\n\n{}\n\nPlease review.",
            meta.to_pr_block()
        );
        let parsed = RemoteStackMeta::from_pr_body(&body).unwrap();
        assert_eq!(parsed, meta);
    }

    #[test]
    fn from_pr_body_returns_none_when_no_block() {
        assert!(RemoteStackMeta::from_pr_body("no block here").is_none());
    }

    #[test]
    fn from_pr_body_ignores_subsequent_code_blocks() {
        let meta = RemoteStackMeta {
            stack_id: StackId("s1".into()),
            frame_id: FrameId("f2".into()),
            position: 1,
            total: 2,
        };
        let body = format!(
            "Description.\n\n{}\n\nExample:\n```rust\nlet x = 1;\n```",
            meta.to_pr_block()
        );
        let parsed = RemoteStackMeta::from_pr_body(&body).unwrap();
        assert_eq!(parsed, meta);
    }
}

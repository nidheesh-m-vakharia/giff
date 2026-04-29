use giff_core::{FrameId, Stack, StackFrame, StackId, StackStore};

fn make_store() -> StackStore {
    StackStore {
        stacks: vec![Stack {
            id: StackId("s1".into()),
            name: "my-stack".into(),
            trunk: "main".into(),
            frames: vec![
                StackFrame {
                    id: FrameId("f1".into()),
                    branch: "feat/a".into(),
                    parent: None,
                    pr_number: None,
                    description: None,
                },
                StackFrame {
                    id: FrameId("f2".into()),
                    branch: "feat/b".into(),
                    parent: Some(FrameId("f1".into())),
                    pr_number: None,
                    description: None,
                },
                StackFrame {
                    id: FrameId("f3".into()),
                    branch: "feat/c".into(),
                    parent: Some(FrameId("f2".into())),
                    pr_number: None,
                    description: None,
                },
            ],
        }],
    }
}

#[test]
fn find_stack_for_branch_finds_middle_frame() {
    let store = make_store();
    let (stack, frame) = store.find_stack_for_branch("feat/b").unwrap();
    assert_eq!(stack.id, StackId("s1".into()));
    assert_eq!(frame.id, FrameId("f2".into()));
}

#[test]
fn find_stack_for_branch_returns_none_for_unknown() {
    let store = make_store();
    assert!(store.find_stack_for_branch("feat/unknown").is_none());
}

#[test]
fn ordered_frames_bottom_to_top() {
    let store = make_store();
    let stack = &store.stacks[0];
    let ordered = stack.ordered_frames();
    let branches: Vec<&str> = ordered.iter().map(|f| f.branch.as_str()).collect();
    assert_eq!(branches, vec!["feat/a", "feat/b", "feat/c"]);
}

#[test]
fn frame_below_returns_parent() {
    let store = make_store();
    let stack = &store.stacks[0];
    let below = stack.frame_below(&FrameId("f2".into())).unwrap();
    assert_eq!(below.id, FrameId("f1".into()));
}

#[test]
fn frame_below_bottom_frame_is_none() {
    let store = make_store();
    let stack = &store.stacks[0];
    assert!(stack.frame_below(&FrameId("f1".into())).is_none());
}

#[test]
fn frame_above_returns_child() {
    let store = make_store();
    let stack = &store.stacks[0];
    let above = stack.frame_above(&FrameId("f1".into())).unwrap();
    assert_eq!(above.id, FrameId("f2".into()));
}

#[test]
fn frame_above_top_frame_is_none() {
    let store = make_store();
    let stack = &store.stacks[0];
    assert!(stack.frame_above(&FrameId("f3".into())).is_none());
}

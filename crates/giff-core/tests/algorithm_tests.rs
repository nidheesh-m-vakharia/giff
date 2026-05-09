use giff_core::{FrameId, Stack, StackFrame, StackId, StackStore};

fn linear_store() -> StackStore {
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

/// Y-shaped stack:
///   root (f1)
///   ├── f2 (left child)
///   └── f3 (right child)
fn y_shaped_stack() -> Stack {
    Stack {
        id: StackId("y1".into()),
        name: "y-stack".into(),
        trunk: "main".into(),
        frames: vec![
            StackFrame {
                id: FrameId("f1".into()),
                branch: "feat/root".into(),
                parent: None,
                pr_number: None,
                description: None,
            },
            StackFrame {
                id: FrameId("f2".into()),
                branch: "feat/left".into(),
                parent: Some(FrameId("f1".into())),
                pr_number: None,
                description: None,
            },
            StackFrame {
                id: FrameId("f3".into()),
                branch: "feat/right".into(),
                parent: Some(FrameId("f1".into())),
                pr_number: None,
                description: None,
            },
        ],
    }
}

#[test]
fn find_stack_for_branch_finds_middle_frame() {
    let store = linear_store();
    let (stack, frame) = store.find_stack_for_branch("feat/b").unwrap();
    assert_eq!(stack.id, StackId("s1".into()));
    assert_eq!(frame.id, FrameId("f2".into()));
}

#[test]
fn find_stack_for_branch_returns_none_for_unknown() {
    let store = linear_store();
    assert!(store.find_stack_for_branch("feat/unknown").is_none());
}

#[test]
fn ordered_frames_linear_returns_bottom_to_top() {
    let store = linear_store();
    let stack = &store.stacks[0];
    let branches: Vec<&str> = stack
        .ordered_frames()
        .iter()
        .map(|f| f.branch.as_str())
        .collect();
    assert_eq!(branches, vec!["feat/a", "feat/b", "feat/c"]);
}

#[test]
fn frame_below_returns_parent() {
    let store = linear_store();
    let stack = &store.stacks[0];
    let below = stack.frame_below(&FrameId("f2".into())).unwrap();
    assert_eq!(below.id, FrameId("f1".into()));
}

#[test]
fn frame_below_root_is_none() {
    let store = linear_store();
    let stack = &store.stacks[0];
    assert!(stack.frame_below(&FrameId("f1".into())).is_none());
}

#[test]
fn children_of_root_in_linear_is_singleton() {
    let store = linear_store();
    let stack = &store.stacks[0];
    let kids = stack.children(&FrameId("f1".into()));
    assert_eq!(kids.len(), 1);
    assert_eq!(kids[0].id, FrameId("f2".into()));
}

#[test]
fn children_of_top_frame_is_empty() {
    let store = linear_store();
    let stack = &store.stacks[0];
    assert!(stack.children(&FrameId("f3".into())).is_empty());
}

#[test]
fn children_of_branching_frame_returns_all() {
    let stack = y_shaped_stack();
    let kids = stack.children(&FrameId("f1".into()));
    assert_eq!(kids.len(), 2);
    let names: Vec<&str> = kids.iter().map(|c| c.branch.as_str()).collect();
    assert!(names.contains(&"feat/left"));
    assert!(names.contains(&"feat/right"));
}

#[test]
fn roots_returns_all_parentless_frames() {
    let stack = Stack {
        id: StackId("multi".into()),
        name: "multi-root".into(),
        trunk: "main".into(),
        frames: vec![
            StackFrame {
                id: FrameId("a".into()),
                branch: "feat/a".into(),
                parent: None,
                pr_number: None,
                description: None,
            },
            StackFrame {
                id: FrameId("b".into()),
                branch: "feat/b".into(),
                parent: None,
                pr_number: None,
                description: None,
            },
        ],
    };
    assert_eq!(stack.roots().len(), 2);
}

#[test]
fn descendants_walks_subtree() {
    let stack = y_shaped_stack();
    let desc = stack.descendants(&FrameId("f1".into()));
    assert_eq!(desc.len(), 2);
    let names: Vec<&str> = desc.iter().map(|d| d.branch.as_str()).collect();
    assert!(names.contains(&"feat/left"));
    assert!(names.contains(&"feat/right"));
}

#[test]
fn descendants_of_leaf_is_empty() {
    let stack = y_shaped_stack();
    assert!(stack.descendants(&FrameId("f2".into())).is_empty());
}

#[test]
fn ancestors_returns_chain_to_root() {
    let store = linear_store();
    let stack = &store.stacks[0];
    let chain = stack.ancestors(&FrameId("f3".into()));
    let names: Vec<&str> = chain.iter().map(|f| f.branch.as_str()).collect();
    assert_eq!(names, vec!["feat/b", "feat/a"]);
}

#[test]
fn ancestors_of_root_is_empty() {
    let store = linear_store();
    let stack = &store.stacks[0];
    assert!(stack.ancestors(&FrameId("f1".into())).is_empty());
}

#[test]
fn path_to_root_includes_self() {
    let store = linear_store();
    let stack = &store.stacks[0];
    let path = stack.path_to_root(&FrameId("f3".into()));
    let names: Vec<&str> = path.iter().map(|f| f.branch.as_str()).collect();
    assert_eq!(names, vec!["feat/a", "feat/b", "feat/c"]);
}

#[test]
fn depth_counts_distance_to_root() {
    let store = linear_store();
    let stack = &store.stacks[0];
    assert_eq!(stack.depth(&FrameId("f1".into())), 0);
    assert_eq!(stack.depth(&FrameId("f2".into())), 1);
    assert_eq!(stack.depth(&FrameId("f3".into())), 2);
}

#[test]
fn topological_order_visits_parents_before_children_in_y_shape() {
    let stack = y_shaped_stack();
    let topo = stack.topological_order();
    let positions: std::collections::HashMap<&str, usize> = topo
        .iter()
        .enumerate()
        .map(|(i, f)| (f.branch.as_str(), i))
        .collect();
    assert!(positions["feat/root"] < positions["feat/left"]);
    assert!(positions["feat/root"] < positions["feat/right"]);
    assert_eq!(topo.len(), 3);
}

#[test]
fn is_linear_true_for_linear_stack() {
    let store = linear_store();
    assert!(store.stacks[0].is_linear());
}

#[test]
fn is_linear_false_for_y_shape() {
    assert!(!y_shaped_stack().is_linear());
}

#[test]
fn validate_accepts_linear() {
    let store = linear_store();
    assert!(store.stacks[0].validate().is_ok());
}

#[test]
fn validate_accepts_tree() {
    assert!(y_shaped_stack().validate().is_ok());
}

#[test]
fn validate_rejects_dangling_parent() {
    let stack = Stack {
        id: StackId("bad".into()),
        name: "bad".into(),
        trunk: "main".into(),
        frames: vec![StackFrame {
            id: FrameId("a".into()),
            branch: "feat/a".into(),
            parent: Some(FrameId("ghost".into())),
            pr_number: None,
            description: None,
        }],
    };
    let err = stack.validate().unwrap_err();
    assert!(format!("{}", err).contains("dangling parent"));
}

#[test]
fn validate_rejects_duplicate_frame_id() {
    let stack = Stack {
        id: StackId("bad".into()),
        name: "bad".into(),
        trunk: "main".into(),
        frames: vec![
            StackFrame {
                id: FrameId("dup".into()),
                branch: "feat/a".into(),
                parent: None,
                pr_number: None,
                description: None,
            },
            StackFrame {
                id: FrameId("dup".into()),
                branch: "feat/b".into(),
                parent: None,
                pr_number: None,
                description: None,
            },
        ],
    };
    let err = stack.validate().unwrap_err();
    assert!(format!("{}", err).contains("duplicate frame id"));
}

#[test]
fn validate_rejects_duplicate_branch() {
    let stack = Stack {
        id: StackId("bad".into()),
        name: "bad".into(),
        trunk: "main".into(),
        frames: vec![
            StackFrame {
                id: FrameId("a".into()),
                branch: "feat/dup".into(),
                parent: None,
                pr_number: None,
                description: None,
            },
            StackFrame {
                id: FrameId("b".into()),
                branch: "feat/dup".into(),
                parent: None,
                pr_number: None,
                description: None,
            },
        ],
    };
    let err = stack.validate().unwrap_err();
    assert!(format!("{}", err).contains("duplicate branch"));
}

#[test]
fn parent_updates_after_pruning_promotes_child_to_root_when_root_merges() {
    // Linear: f1 → f2 → f3. Mark f1 as merged. f2 should become root.
    let store = linear_store();
    let stack = &store.stacks[0];
    let merged: std::collections::HashSet<FrameId> = [FrameId("f1".into())].into_iter().collect();
    let updates = stack.parent_updates_after_pruning(&merged);
    assert_eq!(updates.len(), 1);
    assert_eq!(updates.get(&FrameId("f2".into())).cloned(), Some(None));
}

#[test]
fn parent_updates_after_pruning_walks_through_consecutive_merges() {
    // Linear: f1 → f2 → f3. Mark BOTH f1 and f2 as merged. f3 should become root.
    let store = linear_store();
    let stack = &store.stacks[0];
    let merged: std::collections::HashSet<FrameId> = [FrameId("f1".into()), FrameId("f2".into())]
        .into_iter()
        .collect();
    let updates = stack.parent_updates_after_pruning(&merged);
    assert_eq!(updates.len(), 1);
    assert_eq!(updates.get(&FrameId("f3".into())).cloned(), Some(None));
}

#[test]
fn parent_updates_after_pruning_handles_y_shape_with_root_merged() {
    // Y: f1 root with two children f2, f3. Mark f1 merged. Both children become roots.
    let stack = y_shaped_stack();
    let merged: std::collections::HashSet<FrameId> = [FrameId("f1".into())].into_iter().collect();
    let updates = stack.parent_updates_after_pruning(&merged);
    assert_eq!(updates.len(), 2);
    assert_eq!(updates.get(&FrameId("f2".into())).cloned(), Some(None));
    assert_eq!(updates.get(&FrameId("f3".into())).cloned(), Some(None));
}

#[test]
fn parent_updates_after_pruning_returns_empty_when_no_change() {
    let store = linear_store();
    let stack = &store.stacks[0];
    // Mark f3 (the leaf) as merged — no other frame has it as a parent, so no updates.
    let merged: std::collections::HashSet<FrameId> = [FrameId("f3".into())].into_iter().collect();
    let updates = stack.parent_updates_after_pruning(&merged);
    assert!(updates.is_empty());
}

#[test]
fn validate_rejects_cycle() {
    let stack = Stack {
        id: StackId("bad".into()),
        name: "bad".into(),
        trunk: "main".into(),
        frames: vec![
            StackFrame {
                id: FrameId("a".into()),
                branch: "feat/a".into(),
                parent: Some(FrameId("b".into())),
                pr_number: None,
                description: None,
            },
            StackFrame {
                id: FrameId("b".into()),
                branch: "feat/b".into(),
                parent: Some(FrameId("a".into())),
                pr_number: None,
                description: None,
            },
        ],
    };
    let err = stack.validate().unwrap_err();
    assert!(format!("{}", err).contains("cycle"));
}

use giff_core::{FrameId, Stack, StackFrame, StackId, StackStore};

fn sample_store() -> StackStore {
    StackStore {
        stacks: vec![Stack {
            id: StackId("s1".into()),
            name: "auth-refactor".into(),
            trunk: "main".into(),
            frames: vec![
                StackFrame {
                    id: FrameId("f1".into()),
                    branch: "feat/auth-base".into(),
                    parent: None,
                    pr_number: Some(42),
                    description: None,
                },
                StackFrame {
                    id: FrameId("f2".into()),
                    branch: "feat/auth-tokens".into(),
                    parent: Some(FrameId("f1".into())),
                    pr_number: Some(43),
                    description: None,
                },
            ],
        }],
    }
}

#[test]
fn round_trips_through_toml() {
    let store = sample_store();
    let toml_str = store.to_toml().unwrap();
    let parsed = StackStore::from_toml(&toml_str).unwrap();
    assert_eq!(parsed, store);
}

#[test]
fn from_toml_rejects_invalid_input() {
    let result = StackStore::from_toml("not valid toml ][");
    assert!(result.is_err());
}

#[test]
fn empty_store_round_trips() {
    let store = StackStore { stacks: vec![] };
    let toml_str = store.to_toml().unwrap();
    let parsed = StackStore::from_toml(&toml_str).unwrap();
    assert_eq!(parsed.stacks.len(), 0);
}

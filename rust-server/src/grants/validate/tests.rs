use crate::{Grant, OPERATIONS, Snapshot, sha};
use std::collections::{BTreeMap, HashMap};
fn root() -> Grant {
    let token = "a".repeat(64);
    Grant {
        id: "1".repeat(32),
        parent: None,
        locations: "*".into(),
        operations: OPERATIONS.iter().map(|s| s.to_string()).collect(),
        max_bytes: None,
        max_objects: None,
        issuer_hash: sha(token.as_bytes()),
        root_issuer_token: Some(token),
        users: vec![],
    }
}
#[test]
fn rejects_invalid_root_and_orphans() {
    let g = root();
    let mut grants = HashMap::new();
    grants.insert(g.id.clone(), g.clone());
    let mut state = Snapshot {
        objects: BTreeMap::new(),
        grants,
    };
    state.validate().unwrap();
    state.grants.get_mut(&g.id).unwrap().issuer_hash = "0".repeat(64);
    assert!(state.validate().is_err());
    let mut child = g.clone();
    child.id = "2".repeat(32);
    child.parent = Some("3".repeat(32));
    child.root_issuer_token = None;
    child.max_bytes = Some(2048);
    child.max_objects = Some(1);
    state.grants.insert(child.id.clone(), child);
    assert!(state.validate().is_err());
}

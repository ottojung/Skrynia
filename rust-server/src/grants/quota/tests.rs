use crate::{Grant, OPERATIONS, Snapshot};
use std::collections::{BTreeMap, HashMap};
fn grant(id: &str, parent: Option<&str>, locations: &str) -> Grant {
    Grant {
        id: id.into(),
        parent: parent.map(str::to_string),
        locations: locations.into(),
        operations: OPERATIONS.iter().map(|s| s.to_string()).collect(),
        max_bytes: Some(9000),
        max_objects: Some(4),
        issuer_hash: "0".repeat(64),
        root_issuer_token: None,
        users: vec![],
    }
}
#[test]
fn scoped_quota_and_metadata() {
    let root = "a".repeat(32);
    let child = "b".repeat(32);
    let mut grants = HashMap::new();
    grants.insert(root.clone(), grant(&root, None, "*"));
    grants.insert(child.clone(), grant(&child, Some(&root), "alpha.*"));
    let mut objects = BTreeMap::new();
    objects.insert("alpha.one".into(), 1000);
    objects.insert("beta.one".into(), 9000);
    let data = Snapshot {
        grants,
        objects,
        tokens: HashMap::new(),
    };
    assert_eq!(data.usage(&child, None), (1000 + 512 + 2048, 1));
    assert_eq!(
        data.usage(&child, Some(("alpha.one", 2000))),
        (2000 + 512 + 2048, 1)
    );
    assert_eq!(
        data.usage(&child, Some(("alpha.two", 10))),
        (1000 + 512 + 10 + 512 + 2048, 2)
    );
    assert!(data.check_quota(&child, "alpha.one", 1000).is_err()); // root overlaps a large object
    assert!(data.check_quota(&child, "alpha.two", 9000).is_err());
}

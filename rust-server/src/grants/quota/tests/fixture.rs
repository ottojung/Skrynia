use crate::{Grant, OPERATIONS, QuotaIndex, Snapshot, Usage};
use std::collections::HashMap;
pub fn grant(id: &str, parent: Option<&str>, locations: &str) -> Grant {
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
pub fn snapshot(grants: HashMap<String, Grant>) -> Snapshot {
    let mut quota = QuotaIndex::default();
    for grant in grants.values() {
        quota.scopes.insert(&grant.locations, Usage::default());
        quota.attach_grant(&grant.id, &grant.locations);
        quota.change_metadata(
            &grants,
            &grant.id,
            Usage {
                bytes: 2048,
                objects: 0,
            },
            true,
        );
    }
    Snapshot {
        grants,
        quota,
        tokens: HashMap::new(),
        push: crate::PushState::default(),
    }
}

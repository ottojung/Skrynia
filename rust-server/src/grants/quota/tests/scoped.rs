use super::fixture::{grant, snapshot};
use crate::Snapshot;
use std::collections::HashMap;
pub fn init() -> (Snapshot, String) {
    let root = "a".repeat(32);
    let child = "b".repeat(32);
    let grants = HashMap::from([
        (root.clone(), grant(&root, None, "*")),
        (child.clone(), grant(&child, Some(&root), "alpha.*")),
    ]);
    let mut data = snapshot(grants);
    data.quota.scopes.change("alpha.one", None, Some(1000));
    data.quota.scopes.change("beta.one", None, Some(9000));
    (data, child)
}

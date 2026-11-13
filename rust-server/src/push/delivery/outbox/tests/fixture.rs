use crate::{Pending, PushState, Snapshot};
mod subscription;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::PathBuf,
};
pub fn pending(n: usize) -> (PathBuf, Snapshot, Vec<String>) {
    let root = std::env::temp_dir().join(format!("skrynia-fanout-{}", crate::random_id()));
    fs::create_dir_all(root.join("state/push")).unwrap();
    fs::create_dir_all(root.join("state/push-subs")).unwrap();
    let mut push = PushState::default();
    for index in 0..n {
        let id = format!("{index:032x}");
        let notification = Pending {
            generation: 1,
            attempts: 0,
            next_at: 0,
        };
        push.outbox.insert(id.clone(), notification);
        push.subscriptions
            .insert(id.clone(), subscription::create(&root, &id));
    }
    crate::persist_outbox(&root, &push.outbox).unwrap();
    let ids = push.outbox.keys().cloned().collect();
    let snapshot = Snapshot {
        objects: BTreeMap::new(),
        grants: HashMap::new(),
        tokens: HashMap::new(),
        push,
    };
    (root, snapshot, ids)
}

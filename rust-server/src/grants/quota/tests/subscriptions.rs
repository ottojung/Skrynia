use super::grant;
use crate::Snapshot;
use std::collections::{BTreeMap, HashMap};
#[test]
fn subscription_charges_follow_grant_descendants_only() {
    let root = "a".repeat(32);
    let child = "b".repeat(32);
    let unrelated = "c".repeat(32);
    let grants = HashMap::from([
        (root.clone(), grant(&root, None, "*")),
        (child.clone(), grant(&child, Some(&root), "team.*")),
        (
            unrelated.clone(),
            grant(&unrelated, Some(&root), "unrelated.*"),
        ),
    ]);
    let mut data = Snapshot {
        grants,
        objects: BTreeMap::new(),
        tokens: HashMap::new(),
        push: crate::PushState::default(),
    };
    let create = |id: &str, grant_id: &str| crate::Subscription {
        id: id.into(),
        grant_id: grant_id.into(),
        locations: "*".into(),
        events: vec!["create".into()],
        endpoint: "https://push.example.org/example".into(),
        keys: crate::PushKeys {
            p256dh: "key".into(),
            auth: "auth".into(),
        },
        cap_hash: "f".repeat(64),
    };
    let sub = create(&"d".repeat(32), &child);
    let other = create(&"e".repeat(32), &unrelated);
    data.push.subscriptions.insert(sub.id.clone(), sub);
    data.push.subscriptions.insert(other.id.clone(), other);
    assert_eq!(data.usage(&child, None), (2048 + 4096, 1));
    assert_eq!(data.usage(&unrelated, None), (2048 + 4096, 1));
    assert_eq!(data.usage(&root, None), (3 * 2048 + 2 * 4096, 2));
}

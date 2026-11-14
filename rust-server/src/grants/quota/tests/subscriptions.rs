use super::fixture::{grant, snapshot};
use crate::{Snapshot, Subscription, Usage};
use std::collections::HashMap;
fn sub(id: &str, grant: &str) -> Subscription {
    Subscription {
        id: id.into(),
        grant_id: grant.into(),
        locations: "*".into(),
        events: vec!["create".into()],
        endpoint: "https://push.example.org/example".into(),
        keys: crate::PushKeys {
            p256dh: "BGsX0fLhLEJH-Lzm5WOkQPJ3A32BLeszoPShOUXYmMKWT-NC4v4af5uO5-tKfA-eFivOM1drMV7Oy7ZAaDe_UfU".into(),
            auth: "BwcHBwcHBwcHBwcHBwcHBw".into(),
        },
        cap_hash: "f".repeat(64),
    }
}
fn add(data: &mut Snapshot, id: &str, grant: &str) {
    data.push.put_sub(sub(id, grant));
    data.change_meta(
        grant,
        Usage {
            bytes: 4096,
            objects: 1,
        },
        true,
    );
}
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
    let mut data = snapshot(grants);
    add(&mut data, &"d".repeat(32), &child);
    add(&mut data, &"e".repeat(32), &unrelated);
    assert_eq!(data.usage(&child, None), (2048 + 4096, 1));
    assert_eq!(data.usage(&unrelated, None), (2048 + 4096, 1));
    assert_eq!(data.usage(&root, None), (3 * 2048 + 2 * 4096, 2));
}

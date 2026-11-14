use crate::{PushKeys, Subscription};
use std::{fs, path::Path};
pub fn create(root: &Path, id: &str) -> Subscription {
    let entry = Subscription {
        id: id.into(),
        grant_id: "a".repeat(32),
        locations: "*".into(),
        events: vec!["replace".into()],
        endpoint: "https://push.example.org".into(),
        keys: PushKeys {
            p256dh: "BGsX0fLhLEJH-Lzm5WOkQPJ3A32BLeszoPShOUXYmMKWT-NC4v4af5uO5-tKfA-eFivOM1drMV7Oy7ZAaDe_UfU".into(),
            auth: "BwcHBwcHBwcHBwcHBwcHBw".into(),
        },
        cap_hash: "b".repeat(64),
    };
    fs::write(
        root.join(format!("state/push-subs/{id}.json")),
        serde_json::to_vec(&entry).unwrap(),
    )
    .unwrap();
    entry
}

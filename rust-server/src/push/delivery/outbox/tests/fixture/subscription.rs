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
            p256dh: "x".into(),
            auth: "y".into(),
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

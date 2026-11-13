use crate::{Pending, PushKeys, Subscription, write_json};
use std::fs;
use std::path::Path;
pub fn prepare(path: &Path, generation: u64) -> Subscription {
    for sub in ["state/push-subs", "state/push"] {
        fs::create_dir_all(path.join(sub)).unwrap();
    }
    let id = "a".repeat(32);
    let sub = Subscription {
        id: id.clone(),
        grant_id: "b".repeat(32),
        locations: "*".into(),
        events: vec!["create".into()],
        endpoint: "https://push.example.org/s".into(),
        keys: PushKeys {
            p256dh: "a".into(),
            auth: "b".into(),
        },
        cap_hash: "e".repeat(64),
    };
    let notice = Pending {
        generation,
        attempts: 0,
        next_at: 0,
    };
    write_json(
        &path.join("state/push-subs").join(format!("{id}.json")),
        &sub,
    )
    .unwrap();
    crate::persist_outbox(path, &std::collections::HashMap::from([(id, notice)])).unwrap();
    sub
}

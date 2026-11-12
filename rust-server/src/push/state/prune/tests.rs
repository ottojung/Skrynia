use super::orphans;
use crate::{Notification, PushState, write_json};
use std::fs;
#[test]
fn pending_notifications_without_subscriptions_do_not_consume_capacity() {
    let root = std::env::temp_dir().join(format!("skrynia-outbox-{}", crate::random_id()));
    let dir = root.join("state/push/outbox");
    fs::create_dir_all(&dir).unwrap();
    let id = "a".repeat(32);
    let item = Notification {
        subscription_id: id.clone(),
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    let file = dir.join(format!("{id}.json"));
    write_json(&file, &item).unwrap();
    let mut state = PushState::default();
    state.outbox.insert(id, item);
    orphans(&root, &mut state).unwrap();
    assert!(state.outbox.is_empty());
    assert!(!file.exists());
    assert!(orphans(&root, &mut state).is_ok());
    fs::remove_dir_all(root).unwrap();
}

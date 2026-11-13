use super::read;
use std::fs;
#[test]
fn pending_notifications_without_subscriptions_do_not_consume_capacity() {
    let root = std::env::temp_dir().join(format!("skrynia-orphan-{}", crate::random_id()));
    fs::create_dir_all(root.join("state/push-subs")).unwrap();
    fs::create_dir_all(root.join("state/push")).unwrap();
    let id = "a".repeat(32);
    let notice = crate::Pending {
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    let map = std::collections::HashMap::from([(id.clone(), notice)]);
    crate::persist_outbox(&root, &map).unwrap();
    assert_eq!(read(&root).unwrap().len(), 1);
    assert!(crate::load_push(&root).is_err());
    assert_eq!(
        read(&root).unwrap().len(),
        1,
        "orphan state silently discarded"
    );
    fs::remove_dir_all(root).unwrap();
}

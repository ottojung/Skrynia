use super::fixture::pending;
use std::fs;
#[test]
fn many_first_notifications_use_one_durable_record() {
    let (root, mut data, ids) = pending(50);
    data.push.outbox.clear();
    crate::persist_outbox(&root, &data.push.outbox).unwrap();
    let writes_before = crate::outbox_write_count();
    super::super::enqueue(&mut data, &root, "notice.test", "replace", 100).unwrap();
    assert_eq!(crate::outbox_write_count() - writes_before, 1);
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("state/push/outbox.json")).unwrap()).unwrap();
    assert_eq!(saved.as_object().unwrap().len(), 50);
    assert!(!root.join("state/push/outbox").exists());
    let loaded = crate::load_push(&root).unwrap();
    for id in ids {
        assert_eq!(loaded.outbox[&id].generation, 1);
    }
    fs::remove_dir_all(root).unwrap();
}

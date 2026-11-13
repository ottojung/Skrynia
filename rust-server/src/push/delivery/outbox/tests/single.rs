use super::fixture::pending;
use std::fs;
#[test]
fn one_subscriber_uses_the_canonical_outbox_with_one_write() {
    let (root, mut data, ids) = pending(1);
    let before = crate::outbox_write_count();
    super::super::enqueue(&mut data, &root, "notice.test", "replace", 100).unwrap();
    assert_eq!(crate::outbox_write_count() - before, 1);
    let file = root.join("state/push/outbox.json");
    let records: serde_json::Value = serde_json::from_slice(&fs::read(file).unwrap()).unwrap();
    assert_eq!(records.as_object().unwrap().len(), 1);
    assert_eq!(records[&ids[0]]["generation"], 2);
    assert_eq!(records[&ids[0]]["attempts"], 0);
    assert_eq!(records[&ids[0]]["nextAt"], 0);
    assert!(!root.join("state/push/outbox").exists());
    fs::remove_dir_all(root).unwrap();
}

use super::fixture::pending;
use std::{fs, os::unix::fs::MetadataExt};
#[test]
fn many_pending_subscribers_are_updated_with_one_durable_batch_record() {
    let (root, mut data, ids) = pending(50);
    let path = root.join("state/push/outbox.json");
    let inode = fs::metadata(&path).unwrap().ino();
    let writes_before = crate::outbox_write_count();
    super::super::enqueue(&mut data, &root, "notice.test", "replace", 100).unwrap();
    assert_eq!(crate::outbox_write_count() - writes_before, 1);
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(saved.as_object().unwrap().len(), 50);
    assert_ne!(fs::metadata(&path).unwrap().ino(), inode);
    for id in ids {
        assert_eq!(saved[&id]["generation"], 2);
        assert_eq!(saved[&id]["attempts"], 0);
        assert_eq!(saved[&id]["nextAt"], 0);
        assert_eq!(data.push.outbox[&id].generation, 2);
    }
    assert!(!root.join("state/push/outbox").exists());
    fs::remove_dir_all(root).unwrap();
}

use super::fixture::pending;
use std::fs;
#[test]
fn group_generations_survive_restart_without_recreating_acknowledged_subscribers() {
    let (root, mut data, ids) = pending(50);
    super::super::enqueue(&mut data, &root, "notice.test", "replace", 100).unwrap();
    let before = crate::load_push(&root).unwrap();
    for id in &ids {
        assert_eq!(before.outbox[id].generation, 2);
    }
    let removed = ids[0].clone();
    data.push.outbox.remove(&removed);
    crate::persist_outbox(&root, &data.push.outbox).unwrap();
    let loaded = crate::load_push(&root).unwrap();
    assert!(!loaded.outbox.contains_key(&removed));
    for id in ids.iter().skip(1) {
        assert_eq!(loaded.outbox[id].generation, 2);
    }
    fs::remove_dir_all(root).unwrap();
}

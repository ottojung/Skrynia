use super::fixture::pending;
use std::fs;
#[test]
fn group_generations_survive_restart_without_recreating_acknowledged_subscribers() {
    let (root, mut data, ids) = pending(50);
    super::super::enqueue(&mut data, &root, "notice.test", "replace", 100).unwrap();
    let first = root.join(format!("state/push/outbox/{}.json", ids[1]));
    let base: serde_json::Value = serde_json::from_slice(&fs::read(first).unwrap()).unwrap();
    assert_eq!(
        base["generation"], 1,
        "group update rewrote per-subscriber file"
    );
    let before = crate::load_push(&root).unwrap();
    for id in &ids {
        assert_eq!(before.outbox[id].generation, 2);
    }
    let removed = ids[0].clone();
    fs::remove_file(root.join(format!("state/push/outbox/{removed}.json"))).unwrap();
    data.push.outbox.remove(&removed);
    crate::sync_outbox_index(&data, &root).unwrap();
    let loaded = crate::load_push(&root).unwrap();
    assert!(!loaded.outbox.contains_key(&removed));
    for id in ids.iter().skip(1) {
        assert_eq!(loaded.outbox[id].generation, 2);
    }
    fs::remove_dir_all(root).unwrap();
}

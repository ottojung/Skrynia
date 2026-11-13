use super::fixture::pending;
use std::fs;
#[test]
fn many_first_notifications_use_one_durable_record() {
    let (root, mut data, ids) = pending(50);
    data.push.outbox.clear();
    for id in &ids {
        fs::remove_file(root.join(format!("state/push/outbox/{id}.json"))).unwrap();
    }
    super::super::enqueue(&mut data, &root, "notice.test", "replace", 100).unwrap();
    assert_eq!(
        fs::read_dir(root.join("state/push/outbox"))
            .unwrap()
            .count(),
        0
    );
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("state/push/outbox-generations.json")).unwrap())
            .unwrap();
    assert_eq!(saved.as_object().unwrap().len(), 50);
    let loaded = crate::load_push(&root).unwrap();
    for id in &ids {
        assert_eq!(loaded.outbox[id].generation, 1);
    }
    fs::remove_dir_all(root).unwrap();
}

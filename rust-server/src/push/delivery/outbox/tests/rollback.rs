use super::fixture::pending;
use std::{collections::BTreeMap, fs};
fn generations(data: &crate::Snapshot) -> BTreeMap<String, u64> {
    data.push
        .outbox
        .iter()
        .map(|(id, p)| (id.clone(), p.generation))
        .collect()
}
#[test]
fn failed_persistence_restores_old_and_new_pending_notifications() {
    let (root, mut data, ids) = pending(3);
    data.push.outbox.remove(&ids[0]);
    let before = generations(&data);
    fs::remove_dir_all(root.join("state/push")).unwrap();
    assert!(super::super::enqueue(&mut data, &root, "notice.test", "replace", 10).is_err());
    assert_eq!(generations(&data), before);
    fs::remove_dir_all(root).unwrap();
}

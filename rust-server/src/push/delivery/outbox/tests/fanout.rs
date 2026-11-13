use super::fixture::pending;
use std::{fs, os::unix::fs::MetadataExt};
#[test]
fn many_pending_subscribers_are_updated_with_one_durable_batch_record() {
    let (root, mut data, ids) = pending(50);
    let paths = ids
        .iter()
        .map(|id| root.join(format!("state/push/outbox/{id}.json")))
        .collect::<Vec<_>>();
    let before = paths
        .iter()
        .map(|p| fs::read(p).unwrap())
        .collect::<Vec<_>>();
    let inodes = paths
        .iter()
        .map(|p| fs::metadata(p).unwrap().ino())
        .collect::<Vec<_>>();
    super::super::enqueue(&mut data, &root, "notice.test", "replace", 100).unwrap();
    for ((path, old), inode) in paths.iter().zip(before.iter()).zip(inodes) {
        assert_eq!(&fs::read(path).unwrap(), old, "rewrote pending file");
        assert_eq!(
            fs::metadata(path).unwrap().ino(),
            inode,
            "replaced pending file"
        );
    }
    let batch = root.join("state/push/outbox-generations.json");
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(batch).unwrap()).unwrap();
    for id in &ids {
        assert_eq!(saved[id].as_u64(), Some(2));
        assert_eq!(data.push.outbox[id].generation, 2);
    }
    fs::remove_dir_all(root).unwrap();
}

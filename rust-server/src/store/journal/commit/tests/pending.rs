use super::{commit, root};
use std::{fs, io};
#[test]
fn failed_notification_has_durable_marker_even_if_key_changes_later() {
    let root = root();
    assert!(
        commit(
            &root,
            "test",
            "create",
            Some(crate::sha(b"value")),
            || fs::write(root.join("store/test"), b"value"),
            || Err(io::Error::other("push persistence unavailable"))
        )
        .unwrap()
    );
    let record = fs::read_dir(root.join("state/store-transactions"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(record).unwrap()).unwrap();
    assert_eq!(value["committed"], true);
    fs::write(root.join("store/test"), b"later value").unwrap();
    let mut replayed = 0;
    super::super::super::recover(&root, |_| {
        replayed += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(replayed, 1);
    fs::remove_dir_all(root).unwrap();
}

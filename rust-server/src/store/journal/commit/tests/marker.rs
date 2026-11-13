use super::{commit, root};
use std::{fs, io};
#[test]
fn notification_failure_requires_durable_marker_before_returning_success() {
    let root = root();
    let live = root.join("state/store-transactions");
    let blocked = root.join("state/unavailable");
    let result = commit(
        &root,
        "test",
        "create",
        Some(crate::sha(b"value")),
        || {
            fs::write(root.join("store/test"), b"value")?;
            fs::rename(&live, &blocked)
        },
        || Err(io::Error::other("notification storage unavailable")),
    );
    assert!(result.is_err(), "unsafe success without committed marker");
    fs::rename(&blocked, &live).unwrap();
    let record = fs::read_dir(&live).unwrap().next().unwrap().unwrap().path();
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
    assert_eq!(value["committed"], false);
    let mut replayed = 0;
    super::super::super::recover(&root, |_| {
        replayed += 1;
        Ok(())
    })
    .unwrap();
    assert_eq!(replayed, 1, "initial intent must still recover the change");
    fs::remove_dir_all(root).unwrap();
}

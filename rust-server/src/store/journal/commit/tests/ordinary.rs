use super::{commit, root};
use std::fs;
#[test]
fn ordinary_commits_notify_and_clear_the_initial_intent() {
    let root = root();
    let applied = commit(
        &root,
        "test",
        "create",
        Some(crate::sha(b"value")),
        || fs::write(root.join("store/test"), b"value"),
        || {
            let record = fs::read_dir(root.join("state/store-transactions"))?
                .next()
                .unwrap()?
                .path();
            let value: serde_json::Value = serde_json::from_slice(&fs::read(record)?)?;
            assert_eq!(value["committed"], false);
            Ok(())
        },
    )
    .unwrap();
    assert!(!applied);
    assert_eq!(
        fs::read_dir(root.join("state/store-transactions"))
            .unwrap()
            .count(),
        0
    );
    fs::remove_dir_all(root).unwrap();
}

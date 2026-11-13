use super::{commit, root};
use std::fs;
#[test]
fn failed_initial_intent_write_is_clean_and_never_applies_object() {
    let root = root();
    let dir = root.join("state/store-transactions");
    let moved = root.join("state/transactions-moved");
    fs::rename(&dir, &moved).unwrap();
    fs::write(&dir, b"not a directory").unwrap();
    let result = commit(
        &root,
        "first",
        "create",
        Some(crate::sha(b"x")),
        || panic!("object application must not run"),
        || Ok(()),
    );
    assert!(matches!(result, Err(super::super::Failure::Clean(_))));
    fs::remove_file(dir).unwrap();
    fs::remove_dir_all(root).unwrap();
}

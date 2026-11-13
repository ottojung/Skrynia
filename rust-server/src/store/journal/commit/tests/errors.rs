use super::{commit, root};
use std::{fs, io};
#[test]
fn failed_unapplied_mutation_leaves_no_journal_and_is_retryable() {
    let root = root();
    let result = commit(
        &root,
        "test",
        "create",
        Some(crate::sha(b"payload")),
        || Err(io::Error::from_raw_os_error(libc::ENOSPC)),
        || Ok(()),
    );
    assert!(matches!(result, Err(super::super::Failure::Clean(_))));
    assert_eq!(
        fs::read_dir(root.join("state/store-transactions"))
            .unwrap()
            .count(),
        0
    );
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn failed_applied_mutation_retains_intent_and_requires_recovery() {
    let root = root();
    let result = commit(
        &root,
        "test",
        "create",
        Some(crate::sha(b"payload")),
        || {
            fs::write(root.join("store/test"), b"payload")?;
            Err(io::Error::from_raw_os_error(libc::ENOSPC))
        },
        || Ok(()),
    );
    assert!(matches!(result, Err(super::super::Failure::Uncertain(_))));
    assert_eq!(
        fs::read_dir(root.join("state/store-transactions"))
            .unwrap()
            .count(),
        1
    );
    fs::remove_dir_all(root).unwrap();
}

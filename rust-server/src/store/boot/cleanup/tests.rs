use super::abandoned;
use std::fs;
#[test]
fn remove_only_atomic_writer_orphans() {
    let root = std::env::temp_dir().join(format!("skrynia-orphans-{}", crate::random_id()));
    for dir in [
        "state/grants",
        "state/store-transactions",
        "state/grant-revocations",
        "state/push",
        "state/push-subs",
        "state/push/outbox",
    ] {
        let folder = root.join(dir);
        fs::create_dir_all(&folder).unwrap();
        fs::write(
            folder.join(format!(".skrynia-{}", "a".repeat(32))),
            b"orphan",
        )
        .unwrap();
        fs::write(folder.join(".skrynia-not-an-atomic-id"), b"preserve").unwrap();
    }
    fs::create_dir_all(root.join("store-tmp")).unwrap();
    abandoned(&root).unwrap();
    for dir in [
        "state/grants",
        "state/store-transactions",
        "state/grant-revocations",
        "state/push",
        "state/push-subs",
        "state/push/outbox",
    ] {
        let folder = root.join(dir);
        assert!(!folder.join(format!(".skrynia-{}", "a".repeat(32))).exists());
        assert_eq!(
            fs::read(folder.join(".skrynia-not-an-atomic-id")).unwrap(),
            b"preserve"
        );
    }
    fs::remove_dir_all(root).unwrap();
}

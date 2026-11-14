use crate::{QuotaIndex, Usage};
use std::fs;
#[cfg(unix)]
#[test]
fn quota_enumeration_rejects_nonregular_store_entries() {
    use std::os::unix::fs::symlink;
    let root = std::env::temp_dir().join(format!("skrynia-nonregular-{}", crate::random_id()));
    fs::create_dir_all(root.join("store")).unwrap();
    fs::write(root.join("secret"), b"private").unwrap();
    symlink(root.join("secret"), root.join("store/link")).unwrap();
    assert!(QuotaIndex::scope_usage(&root, "*").is_err());
    let mut index = QuotaIndex::default();
    index.scopes.insert("*", Usage::default());
    assert!(index.scan_objects(&root).is_err());
    fs::remove_dir_all(root).unwrap();
}

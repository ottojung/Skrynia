use super::{ensure_parent, safe_parent, visit};
use std::fs;
#[test]
fn nested_directories_never_follow_symlinks_or_overwrite_files() {
    let root = std::env::temp_dir().join(format!("skrynia-tree-{}", crate::random_id()));
    let store = root.join("store");
    fs::create_dir_all(&store).unwrap();
    ensure_parent(&root, "assets/css/site.css").unwrap();
    assert!(safe_parent(&root, "assets/css/site.css").is_ok());
    fs::write(store.join("file"), b"x").unwrap();
    assert!(ensure_parent(&root, "file/child").is_err());
    let outside = root.join("private");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("secret"), b"secret").unwrap();
    std::os::unix::fs::symlink(&outside, store.join("alias")).unwrap();
    assert!(safe_parent(&root, "alias/secret").is_err());
    assert!(ensure_parent(&root, "alias/new").is_err());
    assert!(visit(&root, |_, _| Ok(())).is_err());
    assert_eq!(fs::read(outside.join("secret")).unwrap(), b"secret");
    fs::remove_dir_all(root).unwrap();
}

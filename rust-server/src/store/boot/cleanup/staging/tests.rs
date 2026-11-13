use super::abandoned;
use std::fs;
#[test]
fn clears_only_verified_abandoned_node_and_rust_staging_files() {
    let root = std::env::temp_dir().join(format!("skrynia-stage-{}", crate::random_id()));
    let folder = root.join("store-tmp");
    fs::create_dir_all(&folder).unwrap();
    let names = [
        format!("obj.123.{}", "a".repeat(24)),
        format!("obj.456.{}", "b".repeat(32)),
    ];
    for name in &names {
        fs::write(folder.join(name), b"abandoned").unwrap();
    }
    for name in ["obj.x.1234", "keep-me", "obj.12.a", "obj.12.z"] {
        fs::write(folder.join(name), b"preserve").unwrap();
    }
    abandoned(&root).unwrap();
    for name in &names {
        assert!(!folder.join(name).exists());
    }
    for name in ["obj.x.1234", "keep-me", "obj.12.a", "obj.12.z"] {
        assert!(folder.join(name).exists());
    }
    fs::remove_dir_all(root).unwrap();
}

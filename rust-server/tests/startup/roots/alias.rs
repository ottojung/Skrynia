use skrynia_server::boot;
use std::{fs, os::unix::fs::symlink};

fn wasm(dir: &std::path::Path) -> std::path::PathBuf {
    let p = dir.join("root.wasm");
    let w = wat::parse_str(include_str!("../../../test-fixtures/root_stub.wat")).unwrap();
    std::fs::write(&p, w).unwrap();
    p
}
#[test]
fn rejects_symlink_alias_and_accepts_siblings() {
    let d = std::env::temp_dir().join(format!(
        "sk-roots-alias-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("real")).unwrap();
    std::fs::create_dir_all(d.join("sib")).unwrap();
    symlink(d.join("real"), d.join("alias")).unwrap();
    let root = wasm(&d);
    assert!(boot::Kernel::start(&root, &d.join("real"), &d.join("alias")).is_err());
    assert!(boot::Kernel::start(&root, &d.join("alias"), &d.join("real")).is_err());
    assert!(boot::Kernel::start(&root, &d.join("real"), &d.join("sib")).is_ok());
    let _ = std::fs::remove_dir_all(d);
}

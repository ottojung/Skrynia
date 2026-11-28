use libskrynia::boot;
use std::os::unix::fs::{PermissionsExt, symlink};

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
    for dir in ["real", "sib"] {
        std::fs::set_permissions(d.join(dir), std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let root = wasm(&d);
    assert!(boot::Kernel::start(&root, &d.join("real"), &d.join("alias")).is_err());
    assert!(boot::Kernel::start(&root, &d.join("alias"), &d.join("real")).is_err());
    assert_eq!(
        std::fs::metadata(d.join("real"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o755
    );
    assert!(boot::Kernel::start(&root, &d.join("real"), &d.join("sib")).is_ok());
    for dir in ["real", "sib"] {
        assert_eq!(
            std::fs::metadata(d.join(dir)).unwrap().permissions().mode() & 0o777,
            0o700
        );
    }
    let _ = std::fs::remove_dir_all(d);
}

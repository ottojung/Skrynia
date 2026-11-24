use libskrynia::boot;
use std::fs;

fn wasm(dir: &std::path::Path) -> std::path::PathBuf {
    let p = dir.join("root.wasm");
    let w = wat::parse_str(include_str!("../../../test-fixtures/root_stub.wat")).unwrap();
    fs::write(&p, w).unwrap();
    p
}
fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "sk-roots-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

#[test]
fn rejects_same_nested_and_reversed_roots() {
    let d = tmp("same-nested");
    let user = d.join("user");
    let op_in_user = user.join("operator");
    let op = d.join("operator");
    let user_in_op = op.join("user");
    fs::create_dir_all(&op_in_user).unwrap();
    fs::create_dir_all(&user_in_op).unwrap();
    let root = wasm(&d);
    assert!(boot::Kernel::start(&root, &user, &user).is_err());
    assert!(boot::Kernel::start(&root, &op_in_user, &user).is_err());
    assert!(boot::Kernel::start(&root, &op, &user_in_op).is_err());
    let _ = fs::remove_dir_all(d);
}

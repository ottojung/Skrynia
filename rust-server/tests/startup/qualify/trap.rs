use libskrynia::boot;
use std::fs;
fn kernel(name: &str, wat: &str) -> (std::path::PathBuf, boot::Kernel) {
    let dir = std::env::temp_dir().join(format!("skrynia-q-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(wat).unwrap()).unwrap();
    let k = boot::Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    (dir, k)
}
#[test]
fn trap_recovery_and_guest_isolation() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(drop(call $r(local.get $i)(i32.const 0)(i32.const 1)))(if(i32.eq(i32.load8_u(i32.const 0))(i32.const 84))(then unreachable))(i32.store8(i32.const 1)(i32.add(i32.load8_u(i32.const 1))(i32.const 1)))(drop(call $w(local.get $o)(i32.const 1)(i32.const 1)))(i32.const 0)))"#;
    let (dir, k) = kernel("trap", root);
    assert!(k.invoke(b"T").is_err());
    assert_eq!(k.invoke(b"O").unwrap(), b"\x01");
    assert_eq!(k.invoke(b"O").unwrap(), b"\x01");
    let _ = fs::remove_dir_all(dir);
}

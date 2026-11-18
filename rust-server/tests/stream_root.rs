use skrynia_server::boot::Kernel;
use std::fs;
fn kernel() -> Kernel {
    let dir = std::env::temp_dir().join(format!("skrynia-input-fd-{}", std::process::id()));
    fs::create_dir_all(dir.join("data")).unwrap();
    let root = r#"(module
      (import "skrynia" "read" (func $r (param i32 i32 i32) (result i32)))
      (import "skrynia" "write" (func $w (param i32 i32 i32) (result i32)))
      (import "skrynia" "seek" (func $s (param i32 i64) (result i32)))
      (import "skrynia" "truncate" (func $t (param i32 i64) (result i32)))
      (import "skrynia" "sync" (func $y (param i32) (result i32)))
      (import "skrynia" "close" (func $c (param i32) (result i32)))
      (memory (export "memory") 1)
      (func (export "skrynia_main") (param $fd i32) (param $o i32) (param i32) (result i32)
        (drop (call $r (local.get $fd) (i32.const -1) (i32.const 3)))
        (drop (call $r (local.get $fd) (local.get $o) (i32.const 3)))
        (i32.store8 (i32.add (local.get $o) (i32.const 3)) (call $w (local.get $fd) (i32.const 0) (i32.const 1)))
        (i32.store8 (i32.add (local.get $o) (i32.const 4)) (call $s (local.get $fd) (i64.const 0)))
        (i32.store8 (i32.add (local.get $o) (i32.const 5)) (call $t (local.get $fd) (i64.const 0)))
        (i32.store8 (i32.add (local.get $o) (i32.const 6)) (call $y (local.get $fd)))
        (drop (call $c (local.get $fd)))
        (i32.store8 (i32.add (local.get $o) (i32.const 7)) (call $r (local.get $fd) (local.get $o) (i32.const 1)))
        (i32.const 8)))"#;
    fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    Kernel::start(&dir.join("root.wasm"), &dir.join("data")).unwrap()
}
#[test]
fn request_fd_is_read_only_and_not_seekable() {
    let k = kernel();
    assert_eq!(
        k.invoke(b"abc").unwrap(),
        b"abc\xfc\xfc\xfc\xfc\xf9".to_vec()
    );
}

use libskrynia::boot;
use std::fs;
fn run(name: &str, root: &str) -> Vec<u8> {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = boot::Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    let out = k.invoke(&[]).unwrap();
    let _ = fs::remove_dir_all(&dir);
    out
}
#[test]
fn cas_host_returns_status_and_observed() {
    let root = r#"(module
(import "skrynia" "write" (func $w (param i32 i32 i32) (result i32)))
(import "skrynia" "open_ram" (func $or (result i32)))
(import "skrynia" "truncate" (func $tr (param i32 i64) (result i32)))
(import "skrynia" "cas_ram_u64" (func $cas (param i32 i64 i64 i64 i32) (result i32)))
(memory (export "memory") 1)
(func (export "skrynia_main") (param i32 i32) (result i32)
  (local $fd i32) (local $s i32)
  (local.set $fd (call $or))
  (drop (call $tr (local.get $fd) (i64.const 8)))
  (local.set $s (call $cas (local.get $fd) (i64.const 0) (i64.const 0) (i64.const 7) (i32.const 64)))
  (i32.store8 (i32.const 80) (i32.add (i32.const 48) (local.get $s)))
  (i64.store (i32.const 81) (i64.load (i32.const 64)))
  (local.set $s (call $cas (local.get $fd) (i64.const 0) (i64.const 0) (i64.const 9) (i32.const 64)))
  (i32.store8 (i32.const 89) (i32.add (i32.const 48) (local.get $s)))
  (i64.store (i32.const 90) (i64.load (i32.const 64)))
  (drop (call $w (local.get 1) (i32.const 80) (i32.const 18)))
  (i32.const 0)))"#;
    let out = run("v3-cas-host", root);
    assert_eq!(out[0], b'0');
    assert_eq!(&out[1..9], &0u64.to_le_bytes());
    assert_eq!(out[9], b'0');
    assert_eq!(&out[10..18], &7u64.to_le_bytes());
}

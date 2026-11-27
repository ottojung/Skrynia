use super::support::start_at;
use std::fs;

const GUEST: &str = r#"(module
 (import "skrynia" "open_disk" (func $od (param i64 i64 i32) (result i32)))
 (import "skrynia" "write1" (func $w1 (param i32 i32) (result i32)))
 (import "skrynia" "write1" (func $wo (param i32 i32) (result i32)))
 (import "skrynia" "close" (func $cl (param i32) (result i32)))
 (import "skrynia" "read1" (func $ri (param i32) (result i32)))
 (memory (export "memory") 1)
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)(local $b i32)(local $op i32)
  (local.set $op (call $ri (local.get $in)))
  (if (i32.eq (local.get $op) (i32.const 87))
   (then (local.set $fd (call $od (i64.const 0) (i64.const 11) (i32.const 7)))
    (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (local.get $fd))))
    (if (i32.ne (call $w1 (local.get $fd) (i32.const 90)) (i32.const 0))
     (then (return (i32.const -1))))
    (drop (call $cl (local.get $fd))) (return (i32.const 0))))
  (local.set $fd (call $od (i64.const 0) (i64.const 11) (i32.const 3)))
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (local.get $fd))))
  (local.set $b (call $ri (local.get $fd)))
  (drop (call $cl (local.get $fd)))
  (drop (call $wo (local.get $out) (local.get $b))) (i32.const 0)))"#;

#[test]
fn vm_persistence_is_genuine_and_root_confined() {
    let dir = std::env::temp_dir().join(format!("k10-vm-{}", rand::random::<u64>()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("root.wasm"), wat::parse_str(GUEST).unwrap()).unwrap();
    let k = start_at(&dir, GUEST);
    k.ram().truncate(64).unwrap();
    k.ram().write(0, b"user/abcdef").unwrap();
    assert_eq!(k.invoke(b"W").unwrap(), Vec::<u8>::new());
    assert_eq!(k.invoke(b"R").unwrap(), vec![90]);
    k.ram().write(0, b"operator/ab").unwrap();
    assert!(k.invoke(b"R").is_err());
    drop(k);
    let k2 = start_at(&dir, GUEST);
    k2.ram().truncate(64).unwrap();
    k2.ram().write(0, b"user/abcdef").unwrap();
    assert_eq!(k2.invoke(b"R").unwrap(), vec![90]);
    fs::remove_dir_all(&dir).unwrap();
}

use super::{dir, kernel};

#[test]
fn bulk_read_write_preflight_and_partial() {
    let d = dir("bulk");
    let root = r#"(module
 (import "skrynia_v3_1" "open_ram" (func $or (result i32)))
 (import "skrynia_v3_1" "truncate" (func $tr (param i32 i64) (result i32)))
 (import "skrynia_v3_1" "seek" (func $sk (param i32 i64) (result i64)))
 (import "skrynia_v3_1" "read" (func $rd (param i32 i64 i64) (result i64)))
 (import "skrynia_v3_1" "write" (func $wr (param i32 i64 i64) (result i64)))
 (import "skrynia_v3_1" "read1" (func $r1 (param i32) (result i32)))
 (import "skrynia_v3_1" "write1" (func $w1 (param i32 i32) (result i32)))
 (memory (export "memory") 1)
 (data (i32.const 0) "ABCDE")
 (data (i32.const 8) "ZZ")
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)
  (local.set $fd (call $or))
  (drop (call $tr (local.get $fd) (i64.const 5)))
  (i64.store (i32.const 16) (call $wr (local.get $fd) (i64.const 0) (i64.const 5)))
  (drop (call $sk (local.get $fd) (i64.const -5)))
  (i64.store (i32.const 24) (call $rd (local.get $fd) (i64.const 65535) (i64.const 2)))
  (i32.store8 (i32.const 32) (call $r1 (local.get $fd)))
  (i64.store (i32.const 40) (call $wr (local.get $fd) (i64.const 65536) (i64.const 1)))
  (i32.store8 (i32.const 33) (call $r1 (local.get $fd)))
  (drop (call $sk (local.get $fd) (i64.const -2)))
  (i64.store (i32.const 48) (call $rd (local.get $fd) (i64.const 100) (i64.const 9)))
  (drop (call $sk (local.get $fd) (i64.const -1)))
  (i64.store (i32.const 56) (call $wr (local.get $fd) (i64.const 8) (i64.const 2)))
  (drop (call $w1 (local.get $out) (i32.wrap_i64 (i64.load (i32.const 16)))))
  (drop (call $w1 (local.get $out) (i32.wrap_i64 (i64.add (i64.load (i32.const 24)) (i64.const 10)))))
  (drop (call $w1 (local.get $out) (i32.load8_u (i32.const 32))))
  (drop (call $w1 (local.get $out) (i32.wrap_i64 (i64.add (i64.load (i32.const 40)) (i64.const 10)))))
  (drop (call $w1 (local.get $out) (i32.load8_u (i32.const 33))))
  (drop (call $w1 (local.get $out) (i32.wrap_i64 (i64.load (i32.const 48)))))
  (drop (call $w1 (local.get $out) (i32.wrap_i64 (i64.load (i32.const 56)))))
  (i32.const 0)))"#;
    let k = kernel(&d, root);
    let out = k.invoke(&[]).unwrap();
    assert_eq!(out[0], 5, "bulk write count");
    assert_eq!(out[1] as i8, 3i8, "read preflight is -7");
    assert_eq!(out[2], b'A', "read preflight no cursor move");
    assert_eq!(out[3] as i8, 3i8, "write preflight is -7");
    assert_eq!(out[4], b'B', "write preflight no cursor move");
    assert_eq!(out[5], 5, "partial read returns available count");
    assert_eq!(out[6], 1, "partial write at end returns count");
    let _ = std::fs::remove_dir_all(&d);
}

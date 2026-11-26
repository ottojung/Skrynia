use super::{dir, kernel};

#[test]
fn byte_io_eof_seek_and_cas_errors() {
    let d = dir("byte-io");
    let root = r#"(module
 (import "skrynia" "open_ram" (func $or (result i32)))
 (import "skrynia" "truncate" (func $tr (param i32 i64) (result i32)))
 (import "skrynia" "read1" (func $r1 (param i32) (result i32)))
 (import "skrynia" "write1" (func $w1 (param i32 i32) (result i32)))
 (import "skrynia" "seek" (func $sk (param i32 i64) (result i64)))
 (import "skrynia" "cas1" (func $cas (param i32 i64 i32 i32) (result i32)))
 (import "skrynia" "write" (func $w (param i32 i32 i32) (result i32)))
 (memory (export "memory") 1)
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)
  (local.set $fd (call $or))
  (if (i32.ne (call $tr (local.get $fd) (i64.const 4)) (i32.const 0)) (then (return (i32.const -1))))
  (if (i32.ne (call $w1 (local.get $fd) (i32.const 65)) (i32.const 0)) (then (return (i32.const -2))))
  (if (i64.ne (call $sk (local.get $fd) (i64.const -1)) (i64.const 0)) (then (return (i32.const -3))))
  (i32.store8 (i32.const 0) (call $r1 (local.get $fd)))
  (i32.store8 (i32.const 1) (call $cas (local.get $fd) (i64.const 0) (i32.const 65) (i32.const 66)))
  (drop (call $sk (local.get $fd) (i64.const -1)))
  (i32.store8 (i32.const 2) (call $r1 (local.get $fd)))
  (drop (call $sk (local.get $fd) (i64.const 3)))
  (i32.store8 (i32.const 3) (call $r1 (local.get $fd)))
  (i32.store8 (i32.const 4) (call $w1 (local.get $fd) (i32.const 300)))
  (i32.store8 (i32.const 5) (i32.wrap_i64 (call $sk (local.get $fd) (i64.const -5))))
  (drop (call $w (local.get $out) (i32.const 0) (i32.const 6)))
  (i32.const 0)))"#;
    let k = kernel(&d, root);
    let out = k.invoke(&[]).unwrap();
    assert_eq!(out, vec![65, 65, 66, 255, 253, 250], "byte I/O results");
    let _ = std::fs::remove_dir_all(&d);
}

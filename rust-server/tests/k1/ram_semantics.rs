use super::{dir, kernel};

#[test]
fn bulk_ram_eof_after_shrink_and_write_at_end() {
    let d = dir("ram-semantics");
    let root = r#"(module
 (import "skrynia_v3_1" "open_ram" (func $or (result i32)))
 (import "skrynia_v3_1" "truncate" (func $tr (param i32 i64) (result i32)))
 (import "skrynia_v3_1" "seek" (func $sk (param i32 i64) (result i64)))
 (import "skrynia_v3_1" "read" (func $rd (param i32 i64 i64) (result i64)))
 (import "skrynia_v3_1" "write" (func $wr (param i32 i64 i64) (result i64)))
 (import "skrynia_v3_1" "write1" (func $w1 (param i32 i32) (result i32)))
 (memory (export "memory") 1)
 (data (i32.const 0) "ZZ")
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)
  (local.set $fd (call $or))
  (drop (call $tr (local.get $fd) (i64.const 4)))
  (drop (call $sk (local.get $fd) (i64.const 2)))
  (drop (call $tr (local.get $fd) (i64.const 1)))
  (i64.store (i32.const 16) (call $rd (local.get $fd) (i64.const 8) (i64.const 2)))
  (drop (call $sk (local.get $fd) (i64.const -1)))
  (i64.store (i32.const 24) (call $wr (local.get $fd) (i64.const 0) (i64.const 2)))
  (drop (call $sk (local.get $fd) (i64.const 1)))
  (i64.store (i32.const 32) (call $wr (local.get $fd) (i64.const 0) (i64.const 1)))
  (drop (call $w1 (local.get $out) (i32.wrap_i64 (i64.load (i32.const 16)))))
  (drop (call $w1 (local.get $out) (i32.and (i32.wrap_i64 (i64.load (i32.const 24))) (i32.const 255))))
  (drop (call $w1 (local.get $out) (i32.and (i32.wrap_i64 (i64.load (i32.const 32))) (i32.const 255))))
  (i32.const 0)))"#;
    let k = kernel(&d, root);
    let out = k.invoke(&[]).unwrap();
    assert_eq!(out[0], 0, "read after shrink returns EOF");
    assert_eq!(out[1] as i8, -7i8, "write at end no progress is -7");
    assert_eq!(out[2] as i8, -7i8, "write after failed seek is -7");
    let _ = std::fs::remove_dir_all(&d);
}

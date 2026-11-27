use super::{dir, kernel};

#[test]
fn random_and_directory_reject_out_of_bounds_buffers() {
    let d = dir("buf-bounds");
    let root = r#"(module
 (import "skrynia_v3_1" "secure_random_bits" (func $rnd (param i64 i64) (result i32)))
 (import "skrynia_v3_1" "open_directory" (func $od (param i64 i64) (result i32)))
 (import "skrynia_v3_1" "next_entry" (func $ne (param i32 i64 i64) (result i32)))
 (import "skrynia_v3_1" "close" (func $cl (param i32) (result i32)))
 (import "skrynia_v3_1" "write1" (func $w1 (param i32 i32) (result i32)))
 (memory (export "memory") 1)
 (data (i32.const 0) "user/.")
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)
  (i32.store8 (i32.const 16) (i32.and (call $rnd (i64.const 65535) (i64.const 16)) (i32.const 255)))
  (local.set $fd (call $od (i64.const 0) (i64.const 6)))
  (i32.store8 (i32.const 17) (i32.and (call $ne (local.get $fd) (i64.const 65535) (i64.const 8)) (i32.const 255)))
  (drop (call $cl (local.get $fd)))
  (drop (call $w1 (local.get $out) (i32.load8_u (i32.const 16))))
  (drop (call $w1 (local.get $out) (i32.load8_u (i32.const 17))))
  (i32.const 0)))"#;
    let k = kernel(&d, root);
    assert_eq!(k.invoke(&[]).unwrap(), vec![249, 249], "bounds are -7");
    let _ = std::fs::remove_dir_all(&d);
}

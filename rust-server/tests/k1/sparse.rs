use super::{dir, kernel};

#[test]
fn wasm_can_access_ram_above_four_gib() {
    let d = dir("sparse-ram");
    let root = r#"(module
      (import "skrynia_v3_1" "open_ram" (func $ram (result i32)))
      (import "skrynia_v3_1" "truncate" (func $t (param i32 i64) (result i32)))
      (import "skrynia_v3_1" "seek" (func $s (param i32 i64) (result i64)))
      (import "skrynia_v3_1" "write1" (func $w (param i32 i32) (result i32)))
      (import "skrynia_v3_1" "cas1" (func $c (param i32 i64 i32 i32) (result i32)))
      (import "skrynia_v3_1" "read1" (func $r (param i32) (result i32)))
      (func (export "skrynia_main") (param i32) (param $out i32) (result i32)
        (local $fd i32)
        (local.set $fd (call $ram))
        (if (i32.ne (call $t (local.get $fd) (i64.const 5368709122)) (i32.const 0))
          (then (return (i32.const -5))))
        (if (i64.ne (call $s (local.get $fd) (i64.const 5368709120)) (i64.const 5368709120))
          (then (return (i32.const -5))))
        (drop (call $w (local.get $fd) (i32.const 42)))
        (if (i32.ne (call $c (local.get $fd) (i64.const 5368709120) (i32.const 42) (i32.const 99)) (i32.const 42))
          (then (return (i32.const -5))))
        (drop (call $s (local.get $fd) (i64.const -1)))
        (drop (call $w (local.get $out) (call $r (local.get $fd))))
        (i32.const 0)))"#;
    let k = kernel(&d, root);
    assert_eq!(k.invoke(b"").unwrap(), vec![99]);
    let _ = std::fs::remove_dir_all(&d);
}

use super::{dir, kernel};

#[test]
fn path_arguments_resolve_from_shared_ram_not_linear_memory() {
    let d = dir("shared-ram");
    let root = r#"(module
 (import "skrynia" "open_disk" (func $od (param i64 i64 i32) (result i32)))
 (import "skrynia" "write1" (func $w1 (param i32 i32) (result i32)))
 (import "skrynia" "sync" (func $sy (param i32) (result i32)))
 (import "skrynia" "close" (func $cl (param i32) (result i32)))
 (memory (export "memory") 1)
 (data (i32.const 0) "wrong.txt")
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)
  (local.set $fd (call $od (i64.const 100) (i64.const 10) (i32.const 7)))
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (local.get $fd))))
  (if (i32.ne (call $w1 (local.get $fd) (i32.const 90)) (i32.const 0)) (then (return (i32.const -1))))
  (if (i32.ne (call $sy (local.get $fd)) (i32.const 0)) (then (return (i32.const -2))))
  (drop (call $cl (local.get $fd)))
  (i32.const 0)))"#;
    let k = kernel(&d, root);
    k.ram().truncate(512).unwrap();
    k.ram().write(100, b"shared.txt").unwrap();
    assert_eq!(k.invoke(&[]).unwrap(), Vec::<u8>::new());
    assert_eq!(std::fs::read(d.join("user/shared.txt")).unwrap(), vec![90]);
    assert!(!d.join("user/wrong.txt").exists());
    let _ = std::fs::remove_dir_all(&d);
}

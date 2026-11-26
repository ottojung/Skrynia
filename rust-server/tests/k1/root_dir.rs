use super::{dir, kernel};

#[test]
fn root_directory_opens_through_shared_ram_and_syncs() {
    let d = dir("root-dir");
    let root = r#"(module
 (import "skrynia" "open_directory" (func $od (param i64 i64) (result i32)))
 (import "skrynia" "sync" (func $sy (param i32) (result i32)))
 (import "skrynia" "close" (func $cl (param i32) (result i32)))
 (import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(drop(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $len))
 (memory (export "memory") 1)
 (data (i32.const 0) "x")
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (local $fd i32)
  (local.set $fd (call $od (i64.const 0) (i64.const 1)))
  (if (i32.lt_s (local.get $fd) (i32.const 0)) (then (return (local.get $fd))))
  (i32.store8 (i32.const 0) (call $sy (local.get $fd)))
  (drop (call $cl (local.get $fd)))
  (drop (call $w (local.get $out) (i32.const 0) (i32.const 1)))
  (i32.const 0)))"#;
    let k = kernel(&d, root);
    k.ram().truncate(64).unwrap();
    k.ram().write(0, b".").unwrap();
    assert_eq!(k.invoke(&[]).unwrap(), vec![0], "root sync status");
    let _ = std::fs::remove_dir_all(&d);
}

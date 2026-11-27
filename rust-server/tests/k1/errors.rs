use super::{dir, kernel};

#[test]
fn descriptor_control_errors_use_target_scalar_codes() {
    let d = dir("control-errors");
    let root = r#"(module
      (import "skrynia_v3_1" "open_ram" (func $ram (result i32)))
      (import "skrynia_v3_1" "close" (func $close (param i32) (result i32)))
      (import "skrynia_v3_1" "sync" (func $sync (param i32) (result i32)))
      (import "skrynia_v3_1" "read1" (func $read (param i32) (result i32)))
      (import "skrynia_v3_1" "write1" (func $write (param i32 i32) (result i32)))
      (import "skrynia_v3_1" "open_disk" (func $open (param i64 i64 i32) (result i32)))
      (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
        (local $fd i32)
        (local.set $fd (call $ram))
        (drop (call $close (local.get $fd)))
        (drop (call $write (local.get $out) (i32.and (call $close (local.get $fd)) (i32.const 255))))
        (drop (call $write (local.get $out) (i32.and (call $sync (local.get $fd)) (i32.const 255))))
        (drop (call $write (local.get $out) (i32.and (call $read (local.get $fd)) (i32.const 255))))
        (drop (call $write (local.get $out) (i32.and (call $sync (local.get $in)) (i32.const 255))))
        (drop (call $write (local.get $out) (i32.and (call $open (i64.const 0) (i64.const 0) (i32.const 32)) (i32.const 255))))
        (i32.const 0)))"#;
    let k = kernel(&d, root);
    assert_eq!(k.invoke(b"").unwrap(), vec![254, 254, 254, 252, 253]);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn bulk_access_mode_errors_leave_scalar_cursors_usable() {
    let d = dir("bulk-mode");
    let root = r#"(module
      (import "skrynia_v3_1" "read" (func $rd (param i32 i64 i64) (result i64)))
      (import "skrynia_v3_1" "write" (func $wr (param i32 i64 i64) (result i64)))
      (import "skrynia_v3_1" "read1" (func $r1 (param i32) (result i32)))
      (import "skrynia_v3_1" "write1" (func $w1 (param i32 i32) (result i32)))
      (memory (export "memory") 1)
      (func (export "skrynia_main") (param $in i32)(param $out i32)(result i32)
        (if (i64.ne (call $rd (local.get $out) (i64.const 0) (i64.const 1)) (i64.const -4))
          (then unreachable))
        (if (i64.ne (call $wr (local.get $in) (i64.const 0) (i64.const 1)) (i64.const -4))
          (then unreachable))
        (drop (call $w1 (local.get $out) (call $r1 (local.get $in))))
        (i32.const 0)))"#;
    let k = kernel(&d, root);
    assert_eq!(
        k.invoke(b"y").unwrap(),
        b"y",
        "bulk mode errors do not move cursors"
    );
    let _ = std::fs::remove_dir_all(&d);
}

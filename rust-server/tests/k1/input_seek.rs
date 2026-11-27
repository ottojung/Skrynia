use super::{dir, kernel};

#[test]
fn input_forward_skip_clamps_at_eof_and_backward_is_denied() {
    let d = dir("input-seek");
    let root = r#"(module
 (import "skrynia_v3_1" "read1" (func $r1 (param i32) (result i32)))
 (import "skrynia_v3_1" "seek" (func $sk (param i32 i64) (result i64)))
 (import "skrynia_v3_1" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
 (memory (export "memory") 1)
 (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
  (if (i64.ne (call $sk (local.get $in) (i64.const 2)) (i64.const 2)) (then (return (i32.const -1))))
  (i32.store8 (i32.const 0) (call $r1 (local.get $in)))
  (drop (call $sk (local.get $in) (i64.const 100)))
  (i32.store8 (i32.const 1) (call $r1 (local.get $in)))
  (i32.store8 (i32.const 2) (i32.wrap_i64 (call $sk (local.get $in) (i64.const -1))))
  (drop (call $w (local.get $out) (i32.const 0) (i32.const 3)))
  (i32.const 0)))"#;
    let k = kernel(&d, root);
    assert_eq!(k.invoke(b"abcdef").unwrap(), vec![99, 255, 252]);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn scalar_only_root_can_echo_and_query_output_cursor() {
    let d = dir("output-seek");
    let root = r#"(module
      (import "skrynia_v3_1" "read1" (func $r (param i32) (result i32)))
      (import "skrynia_v3_1" "write1" (func $w (param i32 i32) (result i32)))
      (import "skrynia_v3_1" "seek" (func $s (param i32 i64) (result i64)))
      (func (export "skrynia_main") (param $in i32) (param $out i32) (result i32)
        (drop (call $w (local.get $out) (call $r (local.get $in))))
        (drop (call $w (local.get $out) (call $r (local.get $in))))
        (if (i64.ne (call $s (local.get $out) (i64.const 0)) (i64.const 2))
          (then (return (i32.const -5))))
        (if (i64.ne (call $s (local.get $out) (i64.const 1)) (i64.const -4))
          (then (return (i32.const -5))))
        (i32.const 0)))"#;
    let k = kernel(&d, root);
    assert_eq!(k.invoke(b"ab").unwrap(), b"ab");
    let _ = std::fs::remove_dir_all(&d);
}

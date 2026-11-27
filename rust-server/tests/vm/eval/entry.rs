use super::call::call;
use super::rig::rig;

const NOOP: &str = "(func $cb (global.set $g(i64.const 1)))";

#[test]
fn missing_entrypoint_is_rejected() {
    assert_eq!(call("no-entry", "(module)", -1, 0, 0, NOOP).0, -3);
}

#[test]
fn no_function_requires_zero_context_args() {
    let child = "(module(func(export \"_start\")))";
    assert_eq!(call("args", child, -1, 1, 0, NOOP).0, -3);
}

#[test]
fn code_outside_linear_memory_is_rejected() {
    let root = r#"(module
(import "skrynia_v3_1" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(import "skrynia_v3_1" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
(memory(export "memory") 1)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
(i64.store(i32.const 0)(call $e(i64.const 1099511627776)(i64.const 4)(i32.const -1)(i32.const 0)(i32.const 0)))
(drop(call $w(local.get $out)(i32.const 0)(i32.const 8)))
(i32.const 0)))"#;
    let r = rig("bounds", root);
    let out = r.kernel.invoke(&[]).unwrap();
    assert_eq!(i64::from_le_bytes(out.try_into().unwrap()), -7);
}

#[test]
fn invalid_negative_selector_is_rejected() {
    let child = r#"(module(func(export "_start")))"#;
    assert_eq!(call("bad-selector", child, -2, 0, 0, NOOP).0, -3);
}

#[test]
fn invalid_entrypoint_cannot_execute_start_side_effects() {
    let child = r#"(module(import "a" "mark" (func $m))
      (func $init (call $m))(start $init)
      (func(export "_start")(param i32)))"#;
    assert_eq!(call("bad-start", child, 0, 0, 0, NOOP), (-3, 0));
}

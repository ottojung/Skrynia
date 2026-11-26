use super::call::call;
use super::rig::{put, rig};

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
fn code_outside_shared_ram_is_rejected() {
    let root = r#"(module
(import "skrynia" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(import "skrynia" "write" (func $w(param i32 i32 i32)(result i32)))
(memory(export "memory") 1)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
(i64.store(i32.const 0)(call $e(i64.const 1099511627776)(i64.const 4)(i32.const -1)(i32.const 0)(i32.const 0)))
(drop(call $w(local.get $out)(i32.const 0)(i32.const 8)))
(i32.const 0)))"#;
    let r = rig("bounds", root);
    put(&r, 0, &[0u8; 8]);
    let out = r.kernel.invoke(&[]).unwrap();
    assert_eq!(i64::from_le_bytes(out.try_into().unwrap()), -6);
}

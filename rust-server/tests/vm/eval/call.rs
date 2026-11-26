use super::rig::{put, rig};

pub fn call(
    name: &str,
    child: &str,
    selector: i32,
    arg0: i32,
    arg1: i32,
    supplied: &str,
) -> (i64, i64) {
    let code = wat::parse_str(child).unwrap();
    let root = format!(
        r#"(module
(import "skrynia" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(import "skrynia" "write" (func $w(param i32 i32 i32)(result i32)))
(table(export "__indirect_function_table") 1 funcref)
(memory(export "memory") 2)
(global $g (mut i64)(i64.const 0))
{supplied}
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
(local $r i64)
(local.set $r(call $e(i64.const 0)(i64.const {len})(i32.const {sel})(i32.const {a0})(i32.const {a1})))
(i64.store(i32.const 128)(local.get $r))
(i64.store(i32.const 136)(global.get $g))
(drop(call $w(local.get $out)(i32.const 128)(i32.const 16)))
(i32.const 0)))"#,
        len = code.len(),
        sel = selector,
        a0 = arg0,
        a1 = arg1,
    );
    let r = rig(name, &root);
    put(&r, 0, &code);
    let out = r.kernel.invoke(&[]).unwrap();
    assert_eq!(out.len(), 16, "eval result must be 16 bytes");
    let status = i64::from_le_bytes(out[0..8].try_into().unwrap());
    let state = i64::from_le_bytes(out[8..16].try_into().unwrap());
    (status, state)
}

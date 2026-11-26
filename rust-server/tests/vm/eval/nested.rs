use super::rig::{put, rig};

#[test]
fn nested_eval_uses_supplied_function_in_originating_context() {
    let inner = wat::parse_str("(module(func(export \"_start\")))").unwrap();
    let outer = wat::parse_str(
        r#"(module
(import "app" "outer" (func $o(param i32 i32)(result i32)))
(func(export "_start")(drop(call $o(i32.const 0)(i32.const 0)))))"#,
    )
    .unwrap();
    let root = format!(
        r#"(module
(import "skrynia" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(drop(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $len))
(table(export "__indirect_function_table") 1 funcref)
(memory(export "memory") 2)
(global $hits (mut i64)(i64.const 0))
(func $cb(param i32 i32)(result i32)
(local $s i64)
(local.set $s(call $e(i64.const 4096)(i64.const {ilen})(i32.const -1)(i32.const 0)(i32.const 0)))
(if(i64.eqz(local.get $s))(then(global.set $hits(i64.add(global.get $hits)(i64.const 1)))))
(i32.const 0))
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
(local $r i64)
(local.set $r(call $e(i64.const 0)(i64.const {olen})(i32.const 0)(i32.const 0)(i32.const 0)))
(i64.store(i32.const 128)(local.get $r))
(i64.store(i32.const 136)(global.get $hits))
(drop(call $w(local.get $out)(i32.const 128)(i32.const 16)))
(i32.const 0)))"#,
        ilen = inner.len(),
        olen = outer.len(),
    );
    let r = rig("nested", &root);
    put(&r, 0, &outer);
    put(&r, 4096, &inner);
    let out = r.kernel.invoke(&[]).unwrap();
    let status = i64::from_le_bytes(out[0..8].try_into().unwrap());
    let hits = i64::from_le_bytes(out[8..16].try_into().unwrap());
    assert_eq!(status, 0);
    assert_eq!(hits, 1, "inner eval must run through the supplied function");
}

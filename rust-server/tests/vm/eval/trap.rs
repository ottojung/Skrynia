use super::rig::{put, rig};

#[test]
fn child_trap_returns_error_and_parent_recovers() {
    let bad = wat::parse_str("(module(func(export \"_start\")unreachable))").unwrap();
    let good = wat::parse_str("(module(func(export \"_start\")))").unwrap();
    let root = format!(
        r#"(module
(import "skrynia" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(drop(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $len))
(table(export "__indirect_function_table") 1 funcref)
(memory(export "memory") 2)
(func $cb)
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
(local $a i64)(local $b i64)
(local.set $a(call $e(i64.const 0)(i64.const {blen})(i32.const 0)(i32.const 0)(i32.const 0)))
(local.set $b(call $e(i64.const 4096)(i64.const {glen})(i32.const -1)(i32.const 0)(i32.const 0)))
(i64.store(i32.const 128)(local.get $a))
(i64.store(i32.const 136)(local.get $b))
(drop(call $w(local.get $out)(i32.const 128)(i32.const 16)))
(i32.const 0)))"#,
        blen = bad.len(),
        glen = good.len(),
    );
    let r = rig("trap", &root);
    put(&r, 0, &bad);
    put(&r, 4096, &good);
    let out = r.kernel.invoke(&[]).unwrap();
    let first = i64::from_le_bytes(out[0..8].try_into().unwrap());
    let second = i64::from_le_bytes(out[8..16].try_into().unwrap());
    assert!(first < 0, "trapping child must report an error");
    assert_eq!(second, 0, "parent must recover and continue");
}

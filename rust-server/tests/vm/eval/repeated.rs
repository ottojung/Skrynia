use super::rig::{put, rig};

#[test]
fn repeated_children_do_not_accumulate_instances() {
    let child = wat::parse_str(
        r#"(module
(import "app" "tick" (func $t))
(func(export "_start")(call $t)))"#,
    )
    .unwrap();
    let root = format!(
        r#"(module
(import "skrynia" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(import "skrynia" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
(table(export "__indirect_function_table") 1 funcref)
(memory(export "memory") 2)
(global $ticks (mut i64)(i64.const 0))
(func $cb (global.set $ticks(i64.add(global.get $ticks)(i64.const 1))))
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
(local $a i64)(local $b i64)
(local.set $a(call $e(i64.const 0)(i64.const {len})(i32.const 0)(i32.const 0)(i32.const 0)))
(local.set $b(call $e(i64.const 0)(i64.const {len})(i32.const 0)(i32.const 0)(i32.const 0)))
(i64.store(i32.const 128)(i64.add(local.get $a)(local.get $b)))
(i64.store(i32.const 136)(global.get $ticks))
(drop(call $w(local.get $out)(i32.const 128)(i32.const 16)))
(i32.const 0)))"#,
        len = child.len(),
    );
    let r = rig("repeated", &root);
    put(&r, 0, &child);
    let out = r.kernel.invoke(&[]).unwrap();
    let statuses = i64::from_le_bytes(out[0..8].try_into().unwrap());
    let ticks = i64::from_le_bytes(out[8..16].try_into().unwrap());
    assert_eq!(statuses, 0);
    assert_eq!(ticks, 2);
    assert_eq!(
        r.kernel.cached_modules(),
        1,
        "compile cache is bounded by bytes"
    );
}

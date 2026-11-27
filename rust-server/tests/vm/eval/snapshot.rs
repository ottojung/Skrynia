use super::rig::{rig, wat_data};

#[test]
fn eval_snapshots_code_from_linear_memory_before_execution() {
    let child_a = wat::parse_str("(module(func(export \"_start\")))").unwrap();
    let child_b = wat::parse_str("(module(func(export \"_start\")unreachable))").unwrap();
    let root = format!(
        r#"(module
(import "skrynia_v3_1" "eval" (func $e(param i64 i64 i32 i32 i32)(result i64)))
(import "skrynia_v3_1" "write1" (func $sk_write1_0 (param i32 i32) (result i32)))(func $w (param $fd i32)(param $src i32)(param $len i32)(result i32)(local $i i32)(local $r i32)(block $done(loop $lp(br_if $done(i32.ge_u(local.get $i)(local.get $len)))(local.set $r(call $sk_write1_0(local.get $fd)(i32.load8_u(i32.add(local.get $src)(local.get $i)))))(if(i32.lt_s(local.get $r)(i32.const 0))(then(return(local.get $r))))(local.set $i(i32.add(local.get $i)(i32.const 1)))(br $lp)))(local.get $i))
(table(export "__indirect_function_table") 1 funcref)
(memory(export "memory") 2)
(func $cb)
(elem(i32.const 0) func $cb)
{da}
{db}
(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)
(local $a i64)(local $b i64)
(local.set $a(call $e(i64.const 0)(i64.const {la})(i32.const -1)(i32.const 0)(i32.const 0)))
(local.set $b(call $e(i64.const 4096)(i64.const {lb})(i32.const -1)(i32.const 0)(i32.const 0)))
(i64.store(i32.const 128)(local.get $a))
(i64.store(i32.const 136)(local.get $b))
(drop(call $w(local.get $out)(i32.const 128)(i32.const 16)))
(i32.const 0)))"#,
        da = wat_data(0, &child_a),
        db = wat_data(4096, &child_b),
        la = child_a.len(),
        lb = child_b.len(),
    );
    let r = rig("snapshot", &root);
    let out = r.kernel.invoke(&[]).unwrap();
    let a = i64::from_le_bytes(out[0..8].try_into().unwrap());
    let b = i64::from_le_bytes(out[8..16].try_into().unwrap());
    assert_eq!(a, 0, "first eval sees its own linear-memory code");
    assert_eq!(b, -5, "second eval sees its own linear-memory code");
}

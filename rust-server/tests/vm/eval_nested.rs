use super::stream::run;
#[test]
fn nested_eval_overflow_preserves_outer_result() {
    let root = r#"(module
(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))
(import "skrynia" "eval_output_len"(func $l(result i32)))
(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 3)
(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)
  (local $r i32)
  (local.set $r(call $e(i32.const 32768)(i32.load(i32.const 104))(i32.const 90000)(i32.const 0)(i32.const -1)(i32.const 95000)(i32.const 1)))
  (i32.store8(i32.const 1)(i32.eq(local.get $r)(i32.const -11)))
  (i32.store8(i32.const 2)(i32.eq(call $l)(i32.const 5)))
  (i32.store8(local.get $dp)(i32.const 7))(i32.const 1))
(elem(i32.const 0) func $cb)
(func(export "skrynia_main")(param $i i32)(param $il i32)(param $o i32)(param $oc i32)(result i32)
  (local $r i32)
  (i32.store(i32.const 104)(local.get $il))
  (local.set $r(call $e(local.get $i)(local.get $il)(i32.const 65536)(i32.const 0)(i32.const 0)(i32.const 70000)(i32.const 1)))
  (i32.store8(local.get $o)(i32.load8_u(i32.const 1)))
  (i32.store8(i32.add(local.get $o)(i32.const 1))(i32.load8_u(i32.const 2)))
  (i32.store8(i32.add(local.get $o)(i32.const 2))(i32.eq(local.get $r)(i32.const -11)))
  (i32.store8(i32.add(local.get $o)(i32.const 3))(i32.eq(call $l)(i32.const 5)))
  (i32.const 4)))"#;
    let child = r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))
(memory(export "memory") 2)(data(i32.const 0) "abc")(data(i32.const 16) "hello")
(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)
  (drop(call $i(i32.const 0)(i32.const 3)(i32.const 64)(i32.const 8)))
  (memory.copy(local.get 2)(i32.const 16)(i32.const 5))(i32.const 5)))"#;
    let out = run("v3-nested-ov", root, &wat::parse_str(child).unwrap()).unwrap();
    assert_eq!(out, vec![1, 1, 1, 1]);
}

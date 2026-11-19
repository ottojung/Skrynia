use super::stream::run;
#[test]
fn root_echoes_input() {
    let root = r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $l i32)(param $o i32)(param $c i32)(result i32)(memory.copy(local.get $o)(local.get $i)(local.get $l))(local.get $l)))"#;
    assert_eq!(run("v3-echo", root, b"hi").unwrap(), b"hi");
}
#[test]
fn eval_rejects_malicious_import() {
    let root = r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $i i32)(param $l i32)(param $o i32)(param $c i32)(result i32)(local $r i32)(local.set $r(call $e(local.get $i)(local.get $l)(i32.const 65536)(i32.const 0)(i32.const -1)(i32.const 70000)(i32.const 16)))(i32.store8(local.get $o)(i32.add(i32.const 48)(i32.eq(local.get $r)(i32.const -1))))(i32.const 1)))"#;
    let child = r#"(module(import "wasi_snapshot_preview1" "fd_write"(func(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(i32.const 0)))"#;
    assert_eq!(
        run("v3-mal", root, &wat::parse_str(child).unwrap()).unwrap(),
        b"1"
    );
}
#[test]
fn nested_eval_depth_exhaustion() {
    let root = r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(call $e(local.get $rp)(local.get $rl)(local.get $rp)(local.get $rl)(i32.const 0)(local.get $dp)(local.get $dc)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $i i32)(param $l i32)(param $o i32)(param $c i32)(result i32)(call $e(local.get $i)(local.get $l)(local.get $i)(local.get $l)(i32.const 0)(local.get $o)(local.get $c))))"#;
    let child = r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(func(export "skrynia_main")(param $i i32)(param $l i32)(param $o i32)(param $c i32)(result i32)(call $i(local.get $i)(local.get $l)(local.get $o)(local.get $c))))"#;
    assert!(run("v3-depth", root, &wat::parse_str(child).unwrap()).is_err());
}
#[test]
fn overflow_no_duplicate_side_effects() {
    let root = r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_output_len"(func $l(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 3)(func $cb(param i32 i32 i32 i32)(result i32)(i32.store8(i32.const 0)(i32.add(i32.load8_u(i32.const 0))(i32.const 1)))(i32.store8(local.get 2)(i32.const 120))(i32.const 1))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $i i32)(param $il i32)(param $o i32)(param $oc i32)(result i32)(local $r i32)(local.set $r(call $e(local.get $i)(local.get $il)(i32.const 65536)(i32.const 0)(i32.const 0)(i32.const 70000)(i32.const 1)))(i32.store8(local.get $o)(i32.eq(local.get $r)(i32.const -11)))(i32.store8(i32.add(local.get $o)(i32.const 1))(i32.eq(call $l)(i32.const 5)))(i32.store8(i32.add(local.get $o)(i32.const 2))(i32.load8_u(i32.const 0)))(i32.const 3)))"#;
    let child = r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "hello")(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(drop(call $i(i32.const 0)(i32.const 0)(i32.const 32)(i32.const 8)))(memory.copy(local.get 2)(i32.const 0)(i32.const 5))(i32.const 5)))"#;
    assert_eq!(
        run("v3-ov-side", root, &wat::parse_str(child).unwrap()).unwrap(),
        vec![1, 1, 1]
    );
}

#[test]
fn sequential_evals() {
    let root = r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(func(export "skrynia_main")(param $i i32)(param $l i32)(param $o i32)(param $c i32)(result i32)(local $n i32)(local.set $n(call $e(local.get $i)(local.get $l)(i32.const 65536)(i32.const 0)(i32.const -1)(local.get $o)(i32.const 8)))(drop(call $e(local.get $i)(local.get $l)(i32.const 65536)(i32.const 0)(i32.const -1)(i32.add(local.get $o)(local.get $n))(i32.const 8)))(i32.mul(local.get $n)(i32.const 2))))"#;
    let child = r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(i32.store8(local.get 2)(i32.const 65))(i32.const 1)))"#;
    assert_eq!(
        run("v3-seq", root, &wat::parse_str(child).unwrap()).unwrap(),
        b"AA"
    );
}

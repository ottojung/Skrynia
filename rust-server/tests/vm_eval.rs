use skrynia_server::boot;
use std::fs;
fn run(name: &str, root: &str, body: &[u8]) -> Result<Vec<u8>, i32> {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    let out = k.invoke(body);
    let _ = fs::remove_dir_all(&dir);
    out
}
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
    let r = run("v3-depth", root, &wat::parse_str(child).unwrap());
    assert!(r.is_err());
}

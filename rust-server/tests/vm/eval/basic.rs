use skrynia_server::boot::Kernel;
use std::fs;

fn kernel(name: &str, root: &str) -> Kernel {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap()
}

#[test]
fn root_echoes_input() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(drop(call $w(local.get $out)(i32.const 0)(local.get $n)))(i32.const 0)))"#;
    assert_eq!(kernel("v3-echo-fd", root).invoke(b"hi").unwrap(), b"hi");
}

#[test]
fn eval_rejects_malicious_import() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local $res i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $res(call $e(i32.const 0)(local.get $n)(local.get $in)(local.get $out)(i32.const -1)))(i32.store8(i32.const 70000)(i32.add(i32.const 48)(i32.eq(local.get $res)(i32.const -1))))(drop(call $w(local.get $out)(i32.const 70000)(i32.const 1)))(i32.const 0)))"#;
    let child = r#"(module(import "wasi_snapshot_preview1" "fd_write"(func(param i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    let out = kernel("v3-mal-fd", root)
        .invoke(&wat::parse_str(child).unwrap())
        .unwrap();
    assert_eq!(out, b"1");
}

#[test]
fn eval_rejects_bad_guest_export() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local $res i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $res(call $e(i32.const 0)(local.get $n)(local.get $in)(local.get $out)(i32.const -1)))(i32.store8(i32.const 70000)(i32.add(i32.const 48)(i32.eq(local.get $res)(i32.const -1))))(drop(call $w(local.get $out)(i32.const 70000)(i32.const 1)))(i32.const 0)))"#;
    let child = r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(i32.const 0)))"#;
    let out = kernel("v3-badexp-fd", root)
        .invoke(&wat::parse_str(child).unwrap())
        .unwrap();
    assert_eq!(out, b"1");
}

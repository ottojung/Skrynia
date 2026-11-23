use skrynia_server::boot::Kernel;
use std::fs;

#[test]
fn guest_fd_directions_enforced() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(call $e(i32.const 0)(local.get $n)(local.get $in)(local.get $out)(i32.const -1))))"#;
    let child = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(i32.store8(i32.const 0)(i32.lt_s(call $w(local.get $in)(i32.const 0)(i32.const 0))(i32.const 0)))(i32.store8(i32.const 1)(i32.lt_s(call $r(local.get $out)(i32.const 8)(i32.const 1))(i32.const 0)))(drop(call $w(local.get $out)(i32.const 0)(i32.const 2)))(i32.const 0)))"#;
    let dir = std::env::temp_dir().join("v3-fd-dir");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    assert_eq!(
        k.invoke(&wat::parse_str(child).unwrap()).unwrap(),
        vec![1, 1]
    );
}

#[test]
fn eval_rejects_privileged_guest_imports() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local $res i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $res(call $e(i32.const 0)(local.get $n)(local.get $in)(local.get $out)(i32.const -1)))(i32.store8(i32.const 70000)(i32.add(i32.const 48)(i32.eq(local.get $res)(i32.const -1))))(drop(call $w(local.get $out)(i32.const 70000)(i32.const 1)))(i32.const 0)))"#;
    let disk = r#"(module(import "skrynia" "open_disk"(func(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    let ram = r#"(module(import "skrynia" "open_ram"(func(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    let dir = std::env::temp_dir().join("v3-priv-guest");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    assert_eq!(k.invoke(&wat::parse_str(disk).unwrap()).unwrap(), b"1");
    assert_eq!(k.invoke(&wat::parse_str(ram).unwrap()).unwrap(), b"1");
}

use libskrynia::boot::Kernel;
use std::fs;

#[test]
fn eval_rejects_privileged_guest_imports() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local $res i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $res(call $e(i32.const 0)(local.get $n)(local.get $in)(local.get $out)(i32.const -1)))(i32.store8(i32.const 70000)(i32.add(i32.const 48)(i32.eq(local.get $res)(i32.const -1))))(drop(call $w(local.get $out)(i32.const 70000)(i32.const 1)))(i32.const 0)))"#;
    let disk = r#"(module(import "skrynia" "open_disk"(func(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    let ram = r#"(module(import "skrynia" "open_ram"(func(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    let dir = std::env::temp_dir().join("v3-priv-guest");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let root = root.replacen(
        "(module",
        r#"(module(import "syscall" "double" (func(param i32)(result i32)))"#,
        1,
    );
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let calls =
        libskrynia::Syscalls::optimized([libskrynia::EquivalentCall::new("double", |_| {
            panic!("child must not call syscall")
        })
        .unwrap()])
        .unwrap();
    let k = Kernel::start_with_syscalls(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
        calls,
    )
    .unwrap();
    assert_eq!(k.invoke(&wat::parse_str(disk).unwrap()).unwrap(), b"1");
    assert_eq!(k.invoke(&wat::parse_str(ram).unwrap()).unwrap(), b"1");
    let custom = r#"(module(import "syscall" "double" (func(param i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    assert_eq!(k.invoke(&wat::parse_str(custom).unwrap()).unwrap(), b"1");
}

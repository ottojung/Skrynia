use skrynia_server::boot;
use std::fs;

fn temp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("skrynia-p1-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("data")).unwrap();
    d
}

fn kernel(dir: &std::path::Path, root: &str, p1: &str) -> boot::Kernel {
    fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(dir.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    boot::Kernel::start_with(
        &dir.join("root.wasm"),
        &dir.join("data"),
        Some(&dir.join("p1.wasm")),
    )
    .unwrap()
}

#[test]
fn root_forwards_streaming_fds_to_program1() {
    let dir = temp("echo");
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    let p1 = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(local $n i32)(local $d i32)(local $wrote i32)(block $done(loop $l(local.set $n(call $r(local.get $i)(i32.const 0)(i32.const 4096)))(br_if $done(i32.eq(local.get $n)(i32.const 0)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $d(i32.const 0))(block $b(loop $w2(br_if $b(i32.eq(local.get $d)(local.get $n)))(local.set $wrote(call $w(local.get $o)(local.get $d)(i32.sub(local.get $n)(local.get $d))))(if(i32.lt_s(local.get $wrote)(i32.const 0))(then(return(local.get $wrote))))(local.set $d(i32.add(local.get $d)(local.get $wrote)))(br $w2)))(br $l)))(i32.const 0)))"#;
    let k = kernel(&dir, root, p1);
    assert_eq!(k.invoke(b"hello-stream").unwrap(), b"hello-stream");
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn program1_rejects_unauthorized_imports() {
    let dir = temp("badimp");
    fs::create_dir_all(dir.join("data")).unwrap();
    let root = r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    let p1 = r#"(module(import "skrynia" "eval"(func(param i32 i32 i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#;
    fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(dir.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    assert!(
        boot::Kernel::start_with(
            &dir.join("root.wasm"),
            &dir.join("data"),
            Some(&dir.join("p1.wasm"))
        )
        .is_err()
    );
    let _ = fs::remove_dir_all(&dir);
}

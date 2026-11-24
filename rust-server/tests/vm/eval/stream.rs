use libskrynia::boot::Kernel;
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
fn streaming_eval_echoes_through_fds() {
    let root = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 65536)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(call $e(i32.const 0)(local.get $n)(local.get $in)(local.get $out)(i32.const -1))))"#;
    let child = r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "ok")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 2)))(i32.const 0)))"#;
    let dir = std::env::temp_dir().join("v3-fd-echo");
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
    let out = k.invoke(&wat::parse_str(child).unwrap()).unwrap();
    assert_eq!(out, b"ok");
}

#[test]
fn guest_handles_short_reads_writes() {
    let guest = r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local $d i32)(block $done(loop $lp(local.set $n(call $r(local.get $in)(i32.const 0)(i32.const 1)))(br_if $done(i32.eqz(local.get $n)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(return(local.get $n))))(local.set $d(i32.const 0))(block $wok(loop $wl(br_if $wok(i32.ge_s(local.get $d)(local.get $n)))(local.set $d(i32.add(local.get $d)(call $w(local.get $out)(local.get $d)(i32.sub(local.get $n)(local.get $d)))))(br $wl)))(br $lp)))(i32.const 0)))"#;
    let wasm = wat::parse_str(guest).unwrap();
    let data: String = wasm.iter().map(|b| format!("\\{:02x}", b)).collect();
    let root = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(data(i32.const 0) "{data}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
        data = data,
        len = wasm.len()
    );
    let k = kernel("v3-short-rw", &root);
    assert_eq!(k.invoke(b"abcdefg").unwrap(), b"abcdefg");
}

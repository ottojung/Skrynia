use skrynia_server::boot::Kernel;
use std::fs;
fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "cap-mode-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user/allowed")).unwrap();
    d
}
#[rustfmt::skip]
#[test]
fn read_only_open_disk_bit_denies_write_mode() {
    let d = dir("deny");
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "allowed/x")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 9)(i32.const 64)(i32.const 64)))(i32.store(i32.const 32)(local.get $n))(drop(call $w(local.get $out)(i32.const 32)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_disk"(func $od(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "allowed")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local.set $fd(call $od(local.get $rp)(local.get $rl)(i32.const 22)))(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))(drop(call $c(local.get $fd)))(i32.const 1))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $ds(i32.const 0)(i32.const 4096)(i32.const 7)(i32.const 1)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert!(code < 0, "write mode must be denied, got {code}");
    assert!(!d.join("user/allowed/x").exists(), "file should not be created");
    let _ = fs::remove_dir_all(d);
}
#[rustfmt::skip]
#[test]
fn explicit_write_bit_allows_write_mode() {
    let d = dir("ok");
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "allowed/x")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 9)(i32.const 64)(i32.const 64)))(i32.store(i32.const 32)(local.get $n))(drop(call $w(local.get $out)(i32.const 32)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_disk"(func $od(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "allowed")(data(i32.const 4128) "X")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local.set $fd(call $od(local.get $rp)(local.get $rl)(i32.const 22)))(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))(drop(call $w(local.get $fd)(i32.const 4128)(i32.const 1)))(drop(call $c(local.get $fd)))(i32.const 1))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $ds(i32.const 0)(i32.const 4096)(i32.const 7)(i32.const 129)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert_eq!(code, 1, "write should succeed, got {code}");
    assert_eq!(fs::read(d.join("user/allowed/x")).unwrap(), b"X");
    let _ = fs::remove_dir_all(d);
}

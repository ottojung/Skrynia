use libskrynia::boot::Kernel;
use std::fs;
fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "cap-fds-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user/allowed")).unwrap();
    d
}
#[rustfmt::skip] #[test]
fn callback_cannot_read_preexisting_root_fd() {
    let d = dir("deny"); fs::write(d.join("operator/secret.txt"), b"LTOPSECRET").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 64)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(i32.store(i32.const 0)(local.get $n))(drop(call $w(local.get $out)(i32.const 0)(i32.const 4)))(return(i32.const 0))))(drop(call $w(local.get $out)(i32.const 64)(local.get $n)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_disk"(func $od(param i64 i64 i32)(result i32)))(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "secret.txt")(data(i32.const 4160) "allowed")(global $fd (mut i32)(i32.const -1))(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(call $r(global.get $fd)(local.get $dp)(local.get $dc)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(global.set $fd(call $od (i64.const 4096) (i64.const 10) (i32.const 1)))(if(i32.lt_s(global.get $fd)(i32.const 0))(then(return(global.get $fd))))(drop(call $ds(i32.const 0)(i32.const 4160)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap(); assert_ne!(out.as_slice(), b"LTOPSECRET");
    let code = i32::from_le_bytes(out[..4].try_into().unwrap()); assert!(code < 0, "expected deny, got {code:?} {out:?}"); let _ = fs::remove_dir_all(d);
}
#[rustfmt::skip] #[test]
fn callback_can_open_new_scoped_user_fd() {
    let d = dir("ok"); fs::write(d.join("user/allowed/f.txt"), b"OK").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "allowed/f.txt")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 13)(i32.const 64)(i32.const 64)))(if(i32.lt_s(local.get $n)(i32.const 0))(then(i32.store(i32.const 32)(local.get $n))(drop(call $w(local.get $out)(i32.const 32)(i32.const 4)))(return(i32.const 0))))(drop(call $w(local.get $out)(i32.const 64)(local.get $n)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_disk"(func $od(param i64 i64 i32)(result i32)))(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "allowed")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local $n i32)(local.set $fd(call $od (i64.extend_i32_u local.get $rp) (i64.extend_i32_u local.get $rl) (i32.const 1)))(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))(local.set $n(call $r(local.get $fd)(local.get $dp)(local.get $dc)))(drop(call $c(local.get $fd)))(local.get $n))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $ds(i32.const 0)(i32.const 4096)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap(); assert_eq!(out, b"OK"); let _ = fs::remove_dir_all(d);
}

#[rustfmt::skip] #[test]
fn callback_cannot_cas_preexisting_root_ram_fd() {
    let d = dir("cas"); let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 64)))(i32.store(i32.const 0)(local.get $n))(drop(call $w(local.get $out)(i32.const 0)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_ram"(func $or(result i32)))(import "skrynia" "cas_ram_u64"(func $cas(param i32 i64 i64 i64 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 4096) "allowed")(global $fd (mut i32)(i32.const -1))(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(call $cas(global.get $fd)(i64.const 0)(i64.const 0)(i64.const 1)(i32.const 128)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(global.set $fd(call $or))(drop(call $ds(i32.const 0)(i32.const 4096)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, len=guest.len());
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap(); let code = i32::from_le_bytes(out.try_into().unwrap());
    assert!(code < 0, "cas on retained root ram fd must be denied, got {code}"); let _ = fs::remove_dir_all(d);
}

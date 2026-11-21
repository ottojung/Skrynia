use skrynia_server::boot::Kernel;
use std::fs;
fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!(
        "cap-take-{name}-{}-{}",
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
fn callback_cannot_smuggle_root_fd_into_child_eval() {
    let d = dir("deny");
    fs::write(d.join("operator/secret.txt"), b"TOP").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 64)))(i32.store(i32.const 0)(local.get $n))(drop(call $w(local.get $out)(i32.const 0)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let child = wat::parse_str(r#"(module(memory(export "memory") 1)(func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#).unwrap();
    let ch: String = child.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_disk"(func $od(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 2048) "{ch}")(data(i32.const 4096) "secret.txt")(data(i32.const 4160) "allowed")(global $fd (mut i32)(i32.const -1))(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $o i32)(local.set $o(call $od(i32.const 4160)(i32.const 7)(i32.const 3)))(if(i32.lt_s(local.get $o)(i32.const 0))(then(return(local.get $o))))(call $e(i32.const 2048)(i32.const {cl})(global.get $fd)(local.get $o)(i32.const -1)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(global.set $fd(call $od(i32.const 4096)(i32.const 10)(i32.const 1)))(if(i32.lt_s(global.get $fd)(i32.const 0))(then(return(global.get $fd))))(drop(call $ds(i32.const 0)(i32.const 4160)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, ch=ch, cl=child.len(), len=guest.len());
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert!(code < 0, "root fd smuggle must be denied, got {code}");
    let _ = fs::remove_dir_all(d);
}

#[rustfmt::skip]
#[test]
fn callback_can_transfer_own_fd_to_child_eval() {
    let d = dir("ok");
    fs::write(d.join("user/allowed/in.txt"), b"A").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 0)(i32.const 64)(i32.const 64)))(i32.store(i32.const 0)(local.get $n))(drop(call $w(local.get $out)(i32.const 0)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let child = wat::parse_str(r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $r(local.get $in)(i32.const 0)(i32.const 1)))(drop(call $w(local.get $out)(i32.const 0)(i32.const 1)))(i32.const 0)))"#).unwrap();
    let ch: String = child.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(r#"(module(import "skrynia" "open_disk"(func $od(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $ds(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $ed(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 2048) "{ch}")(data(i32.const 4096) "allowed/in.txt")(data(i32.const 4160) "allowed/out.txt")(data(i32.const 4200) "allowed")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $i i32)(local $o i32)(local.set $i(call $od(i32.const 4096)(i32.const 14)(i32.const 1)))(if(i32.lt_s(local.get $i)(i32.const 0))(then(return(local.get $i))))(local.set $o(call $od(i32.const 4160)(i32.const 15)(i32.const 7)))(if(i32.lt_s(local.get $o)(i32.const 0))(then(return(local.get $o))))(call $e(i32.const 2048)(i32.const {cl})(local.get $i)(local.get $o)(i32.const -1)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $ds(i32.const 0)(i32.const 4200)(i32.const 7)(i32.const 255)))(call $ed(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#, h=h, ch=ch, cl=child.len(), len=guest.len());
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start(&d.join("root.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert_eq!(code, 0, "authorized transfer should succeed, got {code}");
    let _ = fs::remove_dir_all(d);
}

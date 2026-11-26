use libskrynia::boot::Kernel;
use std::fs;

fn dir(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("cap-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user")).unwrap();
    d
}

#[test]
fn child_cannot_read_operator_file_via_callback() {
    let d = dir("op");
    fs::write(d.join("operator/verifier.bin"), b"SECRET").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "verifier.bin")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 12)(i32.const 64)(i32.const 32)))(i32.store(i32.const 96)(local.get $n))(drop(call $w(local.get $out)(i32.const 96)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(
        r#"(module(import "skrynia" "open_disk"(func $od(param i64 i64 i32)(result i32)))(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local $n i32)(local.set $fd(call $od (i64.extend_i32_u local.get $rp) (i64.extend_i32_u local.get $rl) (i32.const 1)))(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))(local.set $n(call $r(local.get $fd)(local.get $dp)(local.get $dc)))(drop(call $c(local.get $fd)))(local.get $n))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#,
        h = h,
        len = guest.len()
    );
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start_with_roots(&d.join("root.wasm"), &d.join("operator"), &d.join("user"))
        .unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert!(code < 0, "operator file must not be readable, got {code}");
    let _ = fs::remove_dir_all(d);
}

#[test]
fn child_cannot_read_user_sibling_via_callback_without_delegation() {
    let d = dir("sib");
    fs::write(d.join("user/sibling.txt"), b"SIB").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "sibling.txt")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $i(i32.const 0)(i32.const 11)(i32.const 64)(i32.const 16)))(i32.store(i32.const 96)(local.get $n))(drop(call $w(local.get $out)(i32.const 96)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(
        r#"(module(import "skrynia" "open_disk"(func $od(param i64 i64 i32)(result i32)))(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local $n i32)(local.set $fd(call $od (i64.extend_i32_u local.get $rp) (i64.extend_i32_u local.get $rl) (i32.const 1)))(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))(local.set $n(call $r(local.get $fd)(local.get $dp)(local.get $dc)))(drop(call $c(local.get $fd)))(local.get $n))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#,
        h = h,
        len = guest.len()
    );
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start_with_roots(&d.join("root.wasm"), &d.join("operator"), &d.join("user"))
        .unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert!(code < 0, "sibling data must not be readable, got {code}");
    let _ = fs::remove_dir_all(d);
}

use libskrynia::boot::Kernel;
use std::fs;

#[test]
fn explicit_delegation_allows_scoped_user_read_write() {
    let d = std::env::temp_dir().join(format!("cap-del-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user/allowed")).unwrap();
    fs::create_dir_all(d.join("user/secret")).unwrap();
    fs::write(d.join("user/allowed/in.txt"), b"OK").unwrap();
    fs::write(d.join("user/secret/no.txt"), b"NO").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0)"allowed/in.txt")(data(i32.const 32)"secret/no.txt")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local $bad i32)(local.set $n(call $i(i32.const 0)(i32.const 14)(i32.const 64)(i32.const 16)))(if(i32.ne(local.get $n)(i32.const 2))(then(return(i32.const -5))))(local.set $bad(call $i(i32.const 32)(i32.const 13)(i32.const 96)(i32.const 16)))(if(i32.ge_s(local.get $bad)(i32.const 0))(then(return(i32.const -6))))(drop(call $w(local.get $out)(i32.const 64)(i32.const 2)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(
        r#"(module(import "skrynia" "open_disk"(func $od(param i32 i32 i32)(result i32)))(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $c(param i32)(result i32)))(import "skrynia" "eval_delegate"(func $e(param i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $s(param i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 512) "allowed")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $fd i32)(local $n i32)(local.set $fd(call $od(local.get $rp)(local.get $rl)(i32.const 1)))(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(local.get $fd))))(local.set $n(call $r(local.get $fd)(local.get $dp)(local.get $dc)))(drop(call $c(local.get $fd)))(local.get $n))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $s(i32.const 0)(i32.const 512)(i32.const 7)(i32.const 127)))(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#,
        h = h,
        len = guest.len()
    );
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start_with_roots(&d.join("root.wasm"), &d.join("operator"), &d.join("user"))
        .unwrap();
    assert_eq!(k.invoke(b"").unwrap(), b"OK");
    let _ = fs::remove_dir_all(d);
}

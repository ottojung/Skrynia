use skrynia_server::boot::Kernel;
use std::fs;

#[test]
fn delegated_callback_cannot_rename_outside_scope() {
    let d = std::env::temp_dir().join(format!("cap-ren-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user/allowed")).unwrap();
    fs::write(d.join("user/allowed/a.txt"), b"A").unwrap();
    fs::write(d.join("user/outside.txt"), b"O").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "invoke"(func $i(param i32 i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 2)(data(i32.const 0)"allowed/a.txt")(data(i32.const 32)"outside.txt")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(i32.store(i32.const 128)(i32.const 0))(i32.store(i32.const 132)(i32.const 13))(i32.store(i32.const 136)(i32.const 32))(i32.store(i32.const 140)(i32.const 11))(local.set $n(call $i(i32.const 128)(i32.const 16)(i32.const 192)(i32.const 4)))(i32.store(i32.const 160)(local.get $n))(drop(call $w(local.get $out)(i32.const 160)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(
        r#"(module(import "skrynia" "rename"(func $rn(param i32 i32 i32 i32)(result i32)))(import "skrynia" "eval_delegate"(func $e(param i32 i32 i32 i32 i32)(result i32)))(import "skrynia" "fs_delegate_scope"(func $s(param i32 i32 i32 i32)(result i32)))(table(export "__indirect_function_table") 1 funcref)(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 512) "allowed")(func $cb(param $rp i32)(param $rl i32)(param $dp i32)(param $dc i32)(result i32)(local $ap i32)(local $al i32)(local $bp i32)(local $bl i32)(local.set $ap(i32.load(local.get $rp)))(local.set $al(i32.load(i32.add(local.get $rp)(i32.const 4))))(local.set $bp(i32.load(i32.add(local.get $rp)(i32.const 8))))(local.set $bl(i32.load(i32.add(local.get $rp)(i32.const 12))))(call $rn(local.get $ap)(local.get $al)(local.get $bp)(local.get $bl)))(elem(i32.const 0) func $cb)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $s(i32.const 0)(i32.const 512)(i32.const 7)(i32.const 127)))(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const 0))))"#,
        h = h,
        len = guest.len()
    );
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start_with_roots(&d.join("root.wasm"), &d.join("operator"), &d.join("user"))
        .unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert!(
        code < 0,
        "rename outside delegated scope must fail, got {code}"
    );
    assert!(d.join("user/outside.txt").exists());
    let _ = fs::remove_dir_all(d);
}

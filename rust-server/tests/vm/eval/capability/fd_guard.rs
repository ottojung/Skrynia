use libskrynia::boot::Kernel;
use std::fs;

#[test]
fn plain_eval_child_cannot_access_guessed_fd() {
    let d = std::env::temp_dir().join(format!("cap-fd-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("operator")).unwrap();
    fs::create_dir_all(d.join("user")).unwrap();
    fs::write(d.join("user/x.txt"), b"X").unwrap();
    let guest = wat::parse_str(r#"(module(import "skrynia" "read"(func $r(param i32 i32 i32)(result i32)))(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $n i32)(local.set $n(call $r(i32.const 5)(i32.const 64)(i32.const 1)))(i32.store(i32.const 96)(local.get $n))(drop(call $w(local.get $out)(i32.const 96)(i32.const 4)))(i32.const 0)))"#).unwrap();
    let h: String = guest.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(
        r#"(module(import "skrynia" "open_disk"(func $od(param i32 i32 i32)(result i32)))(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 4)(data(i32.const 0) "{h}")(data(i32.const 512) "x.txt")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $fd i32)(local.set $fd(call $od(i32.const 512)(i32.const 5)(i32.const 1)))(drop(local.get $fd))(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1))))"#,
        h = h,
        len = guest.len()
    );
    fs::write(d.join("root.wasm"), wat::parse_str(&root).unwrap()).unwrap();
    let k = Kernel::start_with_roots(&d.join("root.wasm"), &d.join("operator"), &d.join("user"))
        .unwrap();
    let out = k.invoke(b"").unwrap();
    let code = i32::from_le_bytes(out.try_into().unwrap());
    assert_eq!(code, -7, "guessed fd must be rejected as badfd, got {code}");
    let _ = fs::remove_dir_all(d);
}

use skrynia_server::boot::Kernel;
use std::fs;

#[test]
fn descriptor_transfer_is_exclusive() {
    let child = r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "A")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(drop(call $w(local.get $out)(i32.const 0)(i32.const 1)))(i32.const 0)))"#;
    let wasm = wat::parse_str(child).unwrap();
    let hex: String = wasm.iter().map(|b| format!("\\{b:02x}")).collect();
    let root = format!(
        r#"(module(import "skrynia" "eval"(func $e(param i32 i32 i32 i32 i32)(result i32)))(memory(export "memory") 2)(data(i32.const 0) "{hex}")(func(export "skrynia_main")(param $in i32)(param $out i32)(result i32)(local $rc i32)(local.set $rc(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1)))(if(i32.ne(local.get $rc)(i32.const 0))(then(return(local.get $rc))))(local.set $rc(call $e(i32.const 0)(i32.const {len})(local.get $in)(local.get $out)(i32.const -1)))(local.get $rc)))"#,
        hex = hex,
        len = wasm.len()
    );
    let dir = std::env::temp_dir().join(format!("v3-exclusive-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::write(dir.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    let k = Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    let r = k.invoke(b"");
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(r.unwrap_err(), -7);
}

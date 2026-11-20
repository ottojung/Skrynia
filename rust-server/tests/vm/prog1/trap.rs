use skrynia_server::boot;
use std::fs;

#[test]
fn trap_after_partial_output_is_failure() {
    let d = std::env::temp_dir().join(format!("skrynia-p1trap-{}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("data")).unwrap();
    let root = r#"(module(import "skrynia" "run_program1"(func $r(param i32 i32)(result i32)))(memory(export "memory") 1)(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(call $r(local.get $i)(local.get $o))))"#;
    let p1 = r#"(module(import "skrynia" "write"(func $w(param i32 i32 i32)(result i32)))(memory(export "memory") 1)(data(i32.const 0) "ab")(func(export "skrynia_main")(param $i i32)(param $o i32)(result i32)(drop(call $w(local.get $o)(i32.const 0)(i32.const 2)))(unreachable)(i32.const 0)))"#;
    fs::write(d.join("r.wasm"), wat::parse_str(root).unwrap()).unwrap();
    fs::write(d.join("p1.wasm"), wat::parse_str(p1).unwrap()).unwrap();
    let k = boot::Kernel::start_with(&d.join("r.wasm"), &d.join("data"), Some(&d.join("p1.wasm")))
        .unwrap();
    assert!(k.invoke(b"").is_err());
    let _ = fs::remove_dir_all(&d);
}

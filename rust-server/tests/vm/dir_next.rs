use std::fs;
#[test]
fn next_entry_reads_long_name() {
    let n = "a".repeat(255);
    let dir = std::env::temp_dir().join("v3-next-long");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator/d")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("operator/d").join(&n), b"x").unwrap();
    let root = r#"(module(import "skrynia" "open_directory"(func $od(param i32 i32)(result i32)))(import "skrynia" "next_entry"(func $ne(param i32 i32 i32)(result i32)))(import "skrynia" "close"(func $cl(param i32)(result i32)))(memory(export "memory") 2)(data(i32.const 0) "d")(func(export "skrynia_main")(param i32 i32 i32 i32)(result i32)(local $d i32)(local $n i32)(local.set $d (call $od (i32.const 0)(i32.const 1)))(local.set $n (call $ne (local.get $d)(i32.const 256)(i32.const 300)))(drop (call $cl (local.get $d)))(i32.store8 (local.get 2) (i32.eq (local.get $n)(i32.const 255)))(i32.const 1)))"#;
    let wrapped = super::adapter::adapt(root);
    fs::write(dir.join("r.wasm"), wat::parse_str(wrapped).unwrap()).unwrap();
    let k = libskrynia::boot::Kernel::start(
        &dir.join("r.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    k.ram().truncate(16).unwrap();
    k.ram().write(0, b"d").unwrap();
    let out = k.invoke(b"").unwrap();
    assert_eq!(out, vec![1]);
    let _ = fs::remove_dir_all(&dir);
}

use skrynia_server::boot;
use std::fs;

#[test]
fn boot_compiles_empty_module() {
    let dir = std::env::temp_dir().join("skrynia-v3-test-ok");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let wasm = dir.join("ok.wasm");
    let data = dir.join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(&wasm, b"\0asm\x01\0\0\0").unwrap();
    assert!(boot::Kernel::start(&wasm, &data).is_ok());
    let _ = fs::remove_dir_all(&dir);
}

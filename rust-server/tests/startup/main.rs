use skrynia_server::{boot, conf};
use std::ffi::OsString;
use std::fs;

#[test]
fn config_requires_env() {
    assert!(conf::Config::from_lookup(|_| None).is_err());
    let ok = conf::Config::from_lookup(|k| match k {
        "SKRYNIA_ROOT_WASM" => Some(OsString::from("a.wasm")),
        "SKRYNIA_DATA_DIR" => Some(OsString::from("/tmp/d")),
        _ => None,
    });
    assert!(ok.is_ok());
}

#[test]
fn boot_rejects_invalid_wasm() {
    let dir = std::env::temp_dir().join("skrynia-v3-test-bad");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let wasm = dir.join("bad.wasm");
    let data = dir.join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(&wasm, b"not wasm").unwrap();
    assert!(boot::Kernel::start(&wasm, &data).is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn boot_rejects_empty_root_abi() {
    let dir = std::env::temp_dir().join("skrynia-v3-test-ok");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let wasm = dir.join("empty.wasm");
    let data = dir.join("data");
    fs::create_dir_all(&data).unwrap();
    fs::write(&wasm, b"\0asm\x01\0\0\0").unwrap();
    assert!(boot::Kernel::start(&wasm, &data).is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ram_max_limit() {
    let ram = skrynia_server::ram::Ram::new();
    assert!(ram.truncate(64 * 1024 * 1024).is_ok());
    assert!(ram.truncate(64 * 1024 * 1024 + 1).is_err());
}
mod case;
mod exports;
mod imports;

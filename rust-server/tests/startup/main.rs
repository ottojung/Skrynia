use libskrynia::{boot, conf};
use std::ffi::OsString;
use std::fs;

#[test]
fn config_requires_env() {
    assert!(conf::Config::from_lookup(|_| None).is_err());
    let ok = conf::Config::from_lookup(|k| match k {
        "SKRYNIA_ROOT_WASM" => Some(OsString::from("a.wasm")),
        "SKRYNIA_OPERATOR_DIR" => Some(OsString::from("/tmp/op")),
        "SKRYNIA_USER_DIR" => Some(OsString::from("/tmp/user")),
        _ => None,
    });
    assert!(ok.is_ok());
}

#[test]
fn boot_rejects_invalid_wasm() {
    let dir = std::env::temp_dir().join("skrynia-v3-test-bad");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let wasm = dir.join("bad.wasm");
    fs::write(&wasm, b"not wasm").unwrap();
    assert!(boot::Kernel::start(&wasm, &dir.join("operator"), &dir.join("user")).is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn boot_rejects_empty_root_abi() {
    let dir = std::env::temp_dir().join("skrynia-v3-test-ok");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let wasm = dir.join("empty.wasm");
    fs::write(&wasm, b"\0asm\x01\0\0\0").unwrap();
    assert!(boot::Kernel::start(&wasm, &dir.join("operator"), &dir.join("user")).is_err());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ram_max_limit() {
    let ram = libskrynia::ram::Ram::new();
    assert!(ram.truncate(64 * 1024 * 1024).is_ok());
    assert!(ram.truncate(libskrynia::ram::MAX_RAM_BYTES + 1).is_err());
}
mod case;
mod exports;
mod imports;
mod qualify;
mod regular;
mod root_size;
mod roots;

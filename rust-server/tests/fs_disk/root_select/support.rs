use libskrynia::boot;
use std::fs;

pub fn setup(name: &str) -> (std::path::PathBuf, boot::Kernel) {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let wasm = dir.join("r.wasm");
    let code = wat::parse_str(include_str!("../../../test-fixtures/root_stub.wat")).unwrap();
    fs::write(&wasm, code).unwrap();
    let k = boot::Kernel::start(&wasm, &dir.join("operator"), &dir.join("user")).unwrap();
    (dir, k)
}

pub fn start_at(dir: &std::path::Path, guest: &str) -> libskrynia::boot::Kernel {
    let p = dir.join("root.wasm");
    std::fs::write(&p, wat::parse_str(guest).unwrap()).unwrap();
    libskrynia::boot::Kernel::start(&p, &dir.join("operator"), &dir.join("user")).unwrap()
}

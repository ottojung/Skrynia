use skrynia_server::boot;
use std::fs as stdfs;
pub fn setup(name: &str) -> (std::path::PathBuf, boot::Kernel) {
    let dir = std::env::temp_dir().join(name);
    let _ = stdfs::remove_dir_all(&dir);
    stdfs::create_dir_all(dir.join("data")).unwrap();
    let wasm = dir.join("r.wasm");
    let code = wat::parse_str(include_str!("../../test-fixtures/root_stub.wat")).unwrap();
    stdfs::write(&wasm, code).unwrap();
    let k = boot::Kernel::start(&wasm, &dir.join("data")).unwrap();
    (dir, k)
}

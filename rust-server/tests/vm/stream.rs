use skrynia_server::boot;
use std::fs;
pub(super) fn run(name: &str, root: &str, body: &[u8]) -> Result<Vec<u8>, i32> {
    let dir = std::env::temp_dir().join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    let wrapped = super::adapter::adapt(root);
    fs::write(dir.join("r.wasm"), wat::parse_str(wrapped).unwrap()).unwrap();
    let k = boot::Kernel::start(&dir.join("r.wasm"), &dir.join("data")).unwrap();
    let out = k.invoke(body);
    let _ = fs::remove_dir_all(&dir);
    out
}

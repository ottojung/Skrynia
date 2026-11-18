use skrynia_server::boot;
use std::fs;

pub(super) fn check(name: &str, wat: &str) -> bool {
    let dir = std::env::temp_dir().join(format!("skrynia-v3-abi-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("data")).unwrap();
    let bytes = wat::parse_str(wat).unwrap();
    fs::write(dir.join("root.wasm"), bytes).unwrap();
    let result = boot::Kernel::start(&dir.join("root.wasm"), &dir.join("data")).is_ok();
    fs::remove_dir_all(&dir).unwrap();
    result
}

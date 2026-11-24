use libskrynia::boot;
use std::fs;

pub(super) fn check(name: &str, wat: &str) -> bool {
    let dir = std::env::temp_dir().join(format!("skrynia-v3-abi-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let bytes = wat::parse_str(wat).unwrap();
    fs::write(dir.join("root.wasm"), bytes).unwrap();
    let result = boot::Kernel::start(
        &dir.join("root.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .is_ok();
    fs::remove_dir_all(&dir).unwrap();
    result
}

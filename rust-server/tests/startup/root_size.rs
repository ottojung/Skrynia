use libskrynia::boot;
use std::fs;
#[test]
fn root_limit_enforced_before_reading_large_file_or_touching_disk() {
    let dir = std::env::temp_dir().join(format!("skrynia-v3-root-limit-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let root = dir.join("root.wasm");
    let file = fs::File::create(&root).unwrap();
    file.set_len(8 * 1024 * 1024 + 1).unwrap();
    let result = boot::Kernel::start(&root, &dir.join("operator"), &dir.join("user"));
    assert!(matches!(result, Err(e) if e == "root module too large"));
    fs::remove_dir_all(dir).unwrap();
}

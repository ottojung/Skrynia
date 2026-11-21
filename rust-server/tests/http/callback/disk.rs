fn dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "skrynia-callback-disk-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ))
}

#[test]
fn privileged_disk_callback_denied_without_explicit_delegation() {
    use skrynia_server::boot::Kernel;
    let dir = dir("deny");
    std::fs::create_dir_all(dir.join("data")).unwrap();
    let root = wat::parse_str(include_str!(
        "../../../test-fixtures/callback/root_disk_denied.wat"
    ))
    .unwrap();
    let guest = wat::parse_str(include_str!(
        "../../../test-fixtures/callback/guest_disk_denied.wat"
    ))
    .unwrap();
    std::fs::write(dir.join("root.wasm"), root).unwrap();
    std::fs::write(dir.join("data/guest.wasm"), guest).unwrap();
    std::fs::write(dir.join("data/private"), b"Z").unwrap();
    let kernel = Kernel::start(&dir.join("root.wasm"), &dir.join("data")).unwrap();
    let value = kernel.invoke(b"").unwrap();
    let code = i32::from_le_bytes(value.try_into().unwrap());
    assert!(code < 0, "expected denied callback, got {code}");
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn privileged_disk_callback_allowed_with_explicit_delegation() {
    use skrynia_server::boot::Kernel;
    let dir = dir("ok");
    std::fs::create_dir_all(dir.join("data")).unwrap();
    let root = wat::parse_str(include_str!(
        "../../../test-fixtures/callback/root_disk_delegated.wat"
    ))
    .unwrap();
    let guest = wat::parse_str(include_str!(
        "../../../test-fixtures/callback/guest_disk.wat"
    ))
    .unwrap();
    std::fs::write(dir.join("root.wasm"), root).unwrap();
    std::fs::write(dir.join("data/guest.wasm"), guest).unwrap();
    std::fs::write(dir.join("data/private"), b"Z").unwrap();
    let kernel = Kernel::start(&dir.join("root.wasm"), &dir.join("data")).unwrap();
    let value = kernel.invoke(b"").unwrap();
    assert_eq!(value, b"Z");
    std::fs::remove_dir_all(dir).unwrap();
}

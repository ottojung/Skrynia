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
    std::fs::create_dir_all(dir.join("operator")).unwrap();
    std::fs::create_dir_all(dir.join("user")).unwrap();
    let root = wat::parse_str(include_str!(
        "../../../../test-fixtures/callback/root_disk_denied.wat"
    ))
    .unwrap();
    let guest = wat::parse_str(include_str!(
        "../../../../test-fixtures/callback/guest_disk_denied.wat"
    ))
    .unwrap();
    std::fs::write(dir.join("root.wasm"), root).unwrap();
    std::fs::write(dir.join("operator/guest.wasm"), guest).unwrap();
    std::fs::write(dir.join("user/private"), b"Z").unwrap();
    let kernel = Kernel::start(
        &dir.join("root.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap();
    let value = kernel.invoke(b"").unwrap();
    let code = i32::from_le_bytes(value.try_into().unwrap());
    assert!(code < 0, "expected denied callback, got {code}");
    std::fs::remove_dir_all(dir).unwrap();
}

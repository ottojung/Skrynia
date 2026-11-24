fn dir(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "skrynia-callback-disk-{name}-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ))
}
#[test]
fn privileged_disk_callback_allowed_with_explicit_delegation() {
    use libskrynia::boot::Kernel;
    let dir = dir("ok");
    std::fs::create_dir_all(dir.join("operator")).unwrap();
    std::fs::create_dir_all(dir.join("user")).unwrap();
    let root = wat::parse_str(include_str!(
        "../../../../test-fixtures/callback/root_disk_delegated.wat"
    ))
    .unwrap();
    let guest = wat::parse_str(include_str!(
        "../../../../test-fixtures/callback/guest_disk.wat"
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
    assert_eq!(value, b"Z");
    std::fs::remove_dir_all(dir).unwrap();
}

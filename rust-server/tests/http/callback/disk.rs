#[test]
fn ordinary_guest_callback_can_forward_privileged_disk_read() {
    use skrynia_server::boot::Kernel;
    let dir = std::env::temp_dir().join(format!(
        "skrynia-callback-disk-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    std::fs::create_dir_all(dir.join("data")).unwrap();
    let root = wat::parse_str(include_str!(
        "../../../test-fixtures/callback/root_disk.wat"
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

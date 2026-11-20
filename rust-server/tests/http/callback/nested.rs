#[test]
fn nested_guest_cannot_steal_root_disk_fd_after_callback() {
    use skrynia_server::boot::Kernel;
    let dir = std::env::temp_dir().join(format!(
        "skrynia-nested-cap-{}-{}",
        std::process::id(),
        rand::random::<u64>()
    ));
    let data = dir.join("data");
    std::fs::create_dir_all(&data).unwrap();
    let nested = wat::parse_str(
        r#"(module (memory(export "memory") 1)
         (func(export "skrynia_main")(param i32 i32)(result i32)(i32.const 0)))"#,
    )
    .unwrap();
    let hex = nested
        .iter()
        .map(|b| format!("\\{b:02x}"))
        .collect::<String>();
    let root = include_str!("../../../test-fixtures/callback/root_nested.wat")
        .replace("@@MODULE@@", &hex)
        .replace("@@LEN@@", &nested.len().to_string());
    let guest = include_str!("../../../test-fixtures/callback/guest_nested.wat");
    std::fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    std::fs::write(data.join("guest.wasm"), wat::parse_str(guest).unwrap()).unwrap();
    for name in ["private", "nested.in", "nested.out"] {
        std::fs::write(data.join(name), b"Z").unwrap();
    }
    let kernel = Kernel::start(&dir.join("root.wasm"), &data).unwrap();
    assert_eq!(kernel.invoke(b"").unwrap(), b"SAFE");
    std::fs::remove_dir_all(dir).unwrap();
}

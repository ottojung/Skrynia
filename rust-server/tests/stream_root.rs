use libskrynia::boot::Kernel;
use std::fs;
fn kernel() -> Kernel {
    let dir = std::env::temp_dir().join(format!("skrynia-input-fd-{}", std::process::id()));
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    let root = include_str!("../test-fixtures/root_stream_permissions.wat");
    fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    Kernel::start(
        &dir.join("root.wasm"),
        &dir.join("operator"),
        &dir.join("user"),
    )
    .unwrap()
}
#[test]
fn request_fd_is_read_only_and_not_seekable() {
    let k = kernel();
    assert_eq!(
        k.invoke(b"abc").unwrap(),
        [b"abc".as_slice(), &[252; 8]].concat()
    );
}

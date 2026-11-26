use libskrynia::fs::RootKind::User as U;
use libskrynia::{boot, fs};
use std::fs as stdfs;
fn setup(name: &str) -> (std::path::PathBuf, boot::Kernel) {
    let dir = std::env::temp_dir().join(name);
    let _ = stdfs::remove_dir_all(&dir);
    stdfs::create_dir_all(dir.join("operator")).unwrap();
    stdfs::create_dir_all(dir.join("user")).unwrap();
    let wasm = dir.join("r.wasm");
    let code = wat::parse_str(include_str!("../../test-fixtures/root_stub.wat")).unwrap();
    stdfs::write(&wasm, code).unwrap();
    let k = boot::Kernel::start(&wasm, &dir.join("operator"), &dir.join("user")).unwrap();
    (dir, k)
}
#[test]
fn node_ops_and_rename() {
    let (dir, k) = setup("skrynia-fs-node");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    s.mkdir(Some(U), b"user/d", fs::node::MkdirMode::create_if_absent())
        .unwrap();
    assert!(s.is_dir(Some(U), b"user/d").unwrap());
    let fd = s
        .open_disk(
            Some(U),
            &mut fds,
            b"user/d/a",
            fs::disk::DiskMode::read_write(),
            0,
        )
        .unwrap();
    s.write(&mut fds, fd, b"hi").unwrap();
    s.close(&mut fds, fd).unwrap();
    s.rename(Some(U), b"user/d/a", b"user/d/b").unwrap();
    assert!(!s.exists(Some(U), b"user/d/a").unwrap());
    s.delete(Some(U), b"user/d/b").unwrap();
    assert!(!s.exists(Some(U), b"user/d/not-created/leaf").unwrap());
    assert!(s.delete(Some(U), b"user/d/not-created/leaf").is_ok());
    let _ = stdfs::remove_dir_all(&dir);
}

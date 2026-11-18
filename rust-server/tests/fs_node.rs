use skrynia_server::{boot, fs};
use std::fs as stdfs;

fn setup(name: &str) -> (std::path::PathBuf, boot::Kernel) {
    let dir = std::env::temp_dir().join(name);
    let _ = stdfs::remove_dir_all(&dir);
    stdfs::create_dir_all(dir.join("data")).unwrap();
    let wasm = dir.join("r.wasm");
    stdfs::write(&wasm, b"\0asm\x01\0\0\0").unwrap();
    let k = boot::Kernel::start(&wasm, &dir.join("data")).unwrap();
    (dir, k)
}

#[test]
fn node_ops_and_rename() {
    let (dir, k) = setup("skrynia-fs-node");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    s.mkdir(b"d", fs::node::MkdirMode::create_if_absent())
        .unwrap();
    assert!(s.exists(b"d").unwrap());
    assert!(s.is_dir(b"d").unwrap());
    assert!(!s.exists(b"d/nope").unwrap());
    let fd = s
        .open_disk(&mut fds, b"d/a", fs::disk::DiskMode::read_write())
        .unwrap();
    s.write(&mut fds, fd, b"hi").unwrap();
    s.close(&mut fds, fd).unwrap();
    s.rename(b"d/a", b"d/b").unwrap();
    assert!(!s.exists(b"d/a").unwrap());
    assert!(s.exists(b"d/b").unwrap());
    s.delete(b"d/b").unwrap();
    assert!(s.delete(b"d/b").is_ok());
    assert!(!s.exists(b"d/b").unwrap());
    let _ = stdfs::remove_dir_all(&dir);
}

#[test]
fn rejects_bad_and_escaping_paths() {
    let (dir, k) = setup("skrynia-fs-node2");
    let s = k.fs();
    assert!(s.exists(b"/abs").is_err());
    assert!(s.exists(b"../x").is_err());
    assert!(s.exists(b"a/../../b").is_err());
    assert!(s.exists(b"a\0b").is_err());
    assert!(s.exists(b"").is_err());
    let mut fds = fs::table::Table::new();
    assert!(
        s.open_disk(&mut fds, b"../out", fs::disk::DiskMode::read_write())
            .is_err()
    );
    let _ = stdfs::remove_dir_all(&dir);
}

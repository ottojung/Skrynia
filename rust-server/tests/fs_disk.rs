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
fn disk_ops_modes_and_errors() {
    let (dir, k) = setup("skrynia-fs-disk");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    let fd = s
        .open_disk(&mut fds, b"a.txt", fs::disk::DiskMode::read_write())
        .unwrap();
    assert_eq!(s.write(&mut fds, fd, b"hello").unwrap(), 5);
    s.seek(&mut fds, fd, 0).unwrap();
    let mut b = [0u8; 5];
    assert_eq!(s.read(&mut fds, fd, &mut b).unwrap(), 5);
    assert_eq!(&b, b"hello");
    s.truncate_fd(&mut fds, fd, 2).unwrap();
    s.seek(&mut fds, fd, 0).unwrap();
    let mut c = [0u8; 5];
    assert_eq!(s.read(&mut fds, fd, &mut c).unwrap(), 2);
    s.sync(&mut fds, fd).unwrap();
    s.close(&mut fds, fd).unwrap();
    assert!(s.read(&mut fds, fd, &mut b).is_err());
    let r = s
        .open_disk(&mut fds, b"a.txt", fs::disk::DiskMode::read_only())
        .unwrap();
    assert!(s.write(&mut fds, r, b"y").is_err());
    s.mkdir(b"subdir", fs::node::MkdirMode::create_if_absent())
        .unwrap();
    assert!(
        s.open_disk(&mut fds, b"subdir", fs::disk::DiskMode::read_only())
            .is_err()
    );
    let fifo = std::ffi::CString::new(dir.join("data/fifo").to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    assert!(
        s.open_disk(&mut fds, b"fifo", fs::disk::DiskMode::read_only())
            .is_err()
    );
    let _ = stdfs::remove_dir_all(&dir);
}

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
    assert!(
        s.open_disk(b"ro.txt", fs::disk::DiskMode::read_only())
            .is_err()
    );
    let fd = s
        .open_disk(b"a.txt", fs::disk::DiskMode::read_write())
        .unwrap();
    assert_eq!(s.write(fd, b"hello").unwrap(), 5);
    s.seek(fd, 0).unwrap();
    let mut b = [0u8; 5];
    assert_eq!(s.read(fd, &mut b).unwrap(), 5);
    assert_eq!(&b, b"hello");
    s.truncate_fd(fd, 2).unwrap();
    s.seek(fd, 0).unwrap();
    let mut c = [0u8; 5];
    assert_eq!(s.read(fd, &mut c).unwrap(), 2);
    s.sync(fd).unwrap();
    s.close(fd).unwrap();
    assert!(s.read(fd, &mut b).is_err());
    let r = s
        .open_disk(b"a.txt", fs::disk::DiskMode::read_only())
        .unwrap();
    assert!(s.write(r, b"y").is_err());
    let _ = stdfs::remove_dir_all(&dir);
}

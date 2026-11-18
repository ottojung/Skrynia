use skrynia_server::{boot, fs};
use std::fs as stdfs;
fn setup(n: &str) -> (std::path::PathBuf, boot::Kernel) {
    let d = std::env::temp_dir().join(n);
    let _ = stdfs::remove_dir_all(&d);
    stdfs::create_dir_all(d.join("data")).unwrap();
    stdfs::write(d.join("r.wasm"), b"\0asm\x01\0\0\0").unwrap();
    let k = boot::Kernel::start(&d.join("r.wasm"), &d.join("data")).unwrap();
    (d, k)
}
#[test]
fn guard_ops() {
    let (d, k) = setup("skrynia-fs-guard");
    let s = k.fs();
    let fd = s.open_disk(b"a", fs::disk::DiskMode::read_write()).unwrap();
    s.close(fd).unwrap();
    let fd2 = s.open_disk(b"b", fs::disk::DiskMode::read_write()).unwrap();
    assert!(s.read(fd, &mut [0u8; 1]).is_err());
    s.close(fd2).unwrap();
    let out = d.join("outside");
    stdfs::write(&out, b"x").unwrap();
    std::os::unix::fs::symlink(&out, d.join("data/link")).unwrap();
    assert!(
        s.open_disk(b"link", fs::disk::DiskMode::read_only())
            .is_err()
    );
    stdfs::create_dir(d.join("data/reald")).unwrap();
    std::os::unix::fs::symlink("../outside", d.join("data/reald/esc")).unwrap();
    assert!(
        s.open_disk(b"reald/esc", fs::disk::DiskMode::read_only())
            .is_err()
    );
    assert!(k.prepare(10_000).is_ok());
    let _ = stdfs::remove_dir_all(&d);
}

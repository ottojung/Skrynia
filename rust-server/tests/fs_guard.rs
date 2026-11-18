use skrynia_server::boot;
use skrynia_server::fs::disk::DiskMode as M;
use std::fs as stdfs;
fn setup(n: &str) -> (std::path::PathBuf, boot::Kernel) {
    let d = std::env::temp_dir().join(n);
    let _ = stdfs::remove_dir_all(&d);
    stdfs::create_dir_all(d.join("data")).unwrap();
    stdfs::write(d.join("r.wasm"), b"\0asm\x01\0\0\0").unwrap();
    (
        d.clone(),
        boot::Kernel::start(&d.join("r.wasm"), &d.join("data")).unwrap(),
    )
}
#[test]
fn guard_ops() {
    let (d, k) = setup("skrynia-fs-guard");
    let s = k.fs();
    let mut fds = skrynia_server::fs::table::Table::new();
    let fd = s.open_disk(&mut fds, b"a", M::read_write()).unwrap();
    s.close(&mut fds, fd).unwrap();
    let fd2 = s.open_disk(&mut fds, b"b", M::read_write()).unwrap();
    assert!(s.read(&mut fds, fd, &mut [0u8; 1]).is_err());
    s.close(&mut fds, fd2).unwrap();
    stdfs::write(d.join("outside"), b"x").unwrap();
    std::os::unix::fs::symlink(d.join("outside"), d.join("data/link")).unwrap();
    assert!(s.open_disk(&mut fds, b"link", M::read_only()).is_err());
    stdfs::create_dir(d.join("data/reald")).unwrap();
    std::os::unix::fs::symlink("../outside", d.join("data/reald/esc")).unwrap();
    assert!(s.open_disk(&mut fds, b"reald/esc", M::read_only()).is_err());
    assert!(k.prepare(10_000).is_ok());
    let _ = stdfs::remove_dir_all(&d);
}

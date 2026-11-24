use libskrynia::boot;
use libskrynia::fs::disk::DiskMode as M;
use std::fs as stdfs;
fn setup(n: &str) -> (std::path::PathBuf, boot::Kernel) {
    let d = std::env::temp_dir().join(n);
    let _ = stdfs::remove_dir_all(&d);
    stdfs::create_dir_all(d.join("operator")).unwrap();
    stdfs::create_dir_all(d.join("user")).unwrap();
    let code = wat::parse_str(include_str!("../test-fixtures/root_stub.wat")).unwrap();
    stdfs::write(d.join("r.wasm"), code).unwrap();
    let k = boot::Kernel::start(&d.join("r.wasm"), &d.join("operator"), &d.join("user")).unwrap();
    (d, k)
}
#[rustfmt::skip]
#[test]
fn guard_ops() {
    let (d, k) = setup("skrynia-fs-guard");
    let s = k.fs();
    let (mut fds, u) = (libskrynia::fs::table::Table::new(), Some(libskrynia::fs::RootKind::User));
    let fd = s.open_disk(u, &mut fds, b"a", M::read_write(), 0).unwrap();
    s.close(&mut fds, fd).unwrap();
    assert_eq!(s.open_disk(u, &mut fds, b"a", M::create_new_rw(), 0).err(), Some("exists"));
    let fd2 = s.open_disk(u, &mut fds, b"b", M::read_write(), 0).unwrap();
    assert!(s.read(&mut fds, fd, &mut [0u8; 1]).is_err());
    s.close(&mut fds, fd2).unwrap();
    stdfs::write(d.join("outside"), b"x").unwrap();
    std::os::unix::fs::symlink(d.join("outside"), d.join("user/link")).unwrap();
    assert!(s.open_disk(u, &mut fds, b"link", M::read_only(), 0).is_err());
    stdfs::create_dir(d.join("user/reald")).unwrap();
    std::os::unix::fs::symlink("../outside", d.join("user/reald/esc")).unwrap();
    assert!(s.open_disk(u, &mut fds, b"reald/esc", M::read_only(), 0).is_err());
    let mode = libskrynia::fs::node::MkdirMode::create_if_absent();
    assert!(s.mkdir(u, b"reald", mode).is_ok());
    assert!(s.mkdir(u, b"a", mode).is_err());
    assert!(s.mkdir(u, b"link", mode).is_err());
    stdfs::write(d.join("user/plain"), b"file").unwrap();
    assert!(s.mkdir(u, b"plain", mode).is_err());
    let new = libskrynia::fs::node::MkdirMode::create_new();
    assert_eq!(s.mkdir(u, b"reald", new).err(), Some("exists"));
    assert!(k.prepare(10_000).is_ok());
    let _ = stdfs::remove_dir_all(&d);
}

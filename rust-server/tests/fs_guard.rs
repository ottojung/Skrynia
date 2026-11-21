use skrynia_server::boot;
use skrynia_server::fs::disk::DiskMode as M;
use std::fs as stdfs;
fn setup(n: &str) -> (std::path::PathBuf, boot::Kernel) {
    let d = std::env::temp_dir().join(n);
    let _ = stdfs::remove_dir_all(&d);
    stdfs::create_dir_all(d.join("data")).unwrap();
    let code = wat::parse_str(include_str!("../test-fixtures/root_stub.wat")).unwrap();
    stdfs::write(d.join("r.wasm"), code).unwrap();
    let k = boot::Kernel::start(&d.join("r.wasm"), &d.join("data")).unwrap();
    (d, k)
}
#[test]
fn guard_ops() {
    let (d, k) = setup("skrynia-fs-guard");
    let s = k.fs();
    let mut fds = skrynia_server::fs::table::Table::new();
    let fd = s.open_disk(None, &mut fds, b"a", M::read_write()).unwrap();
    s.close(&mut fds, fd).unwrap();
    assert_eq!(
        s.open_disk(None, &mut fds, b"a", M::create_new_rw()).err(),
        Some("exists")
    );
    let fd2 = s.open_disk(None, &mut fds, b"b", M::read_write()).unwrap();
    assert!(s.read(&mut fds, fd, &mut [0u8; 1]).is_err());
    s.close(&mut fds, fd2).unwrap();
    stdfs::write(d.join("outside"), b"x").unwrap();
    std::os::unix::fs::symlink(d.join("outside"), d.join("data/link")).unwrap();
    assert!(
        s.open_disk(None, &mut fds, b"link", M::read_only())
            .is_err()
    );
    stdfs::create_dir(d.join("data/reald")).unwrap();
    std::os::unix::fs::symlink("../outside", d.join("data/reald/esc")).unwrap();
    assert!(
        s.open_disk(None, &mut fds, b"reald/esc", M::read_only())
            .is_err()
    );
    let mode = skrynia_server::fs::node::MkdirMode::create_if_absent();
    assert!(s.mkdir(None, b"reald", mode).is_ok());
    assert!(s.mkdir(None, b"a", mode).is_err());
    assert!(s.mkdir(None, b"link", mode).is_err());
    stdfs::write(d.join("data/plain"), b"file").unwrap();
    assert!(s.mkdir(None, b"plain", mode).is_err());
    let new = skrynia_server::fs::node::MkdirMode::create_new();
    assert_eq!(s.mkdir(None, b"reald", new).err(), Some("exists"));
    assert!(k.prepare(10_000).is_ok());
    let _ = stdfs::remove_dir_all(&d);
}

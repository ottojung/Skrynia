mod setup;
use setup::setup;
mod private;
mod ram;
mod safe_path;
mod v3_codes;
mod v3_eval;
mod v3_fixtures;
mod v3_nested;
mod v3_ram;
use skrynia_server::fs::{self, disk::DiskMode};
#[test]
fn disk_ops_modes_and_errors() {
    let (dir, k) = setup("skrynia-fs-disk");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    let u = Some(fs::RootKind::User);
    let fd = s
        .open_disk(u, &mut fds, b"a.txt", DiskMode::read_write(), 0)
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
        .open_disk(u, &mut fds, b"a.txt", DiskMode::read_only(), 0)
        .unwrap();
    assert!(s.write(&mut fds, r, b"y").is_err());
    s.mkdir(u, b"subdir", fs::node::MkdirMode::create_if_absent())
        .unwrap();
    assert!(
        s.open_disk(u, &mut fds, b"subdir", DiskMode::read_only(), 0)
            .is_err()
    );
    let fifo = std::ffi::CString::new(dir.join("user/fifo").to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(fifo.as_ptr(), 0o600) }, 0);
    assert!(
        s.open_disk(u, &mut fds, b"fifo", DiskMode::read_only(), 0)
            .is_err()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

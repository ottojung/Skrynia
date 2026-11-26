use super::support::setup;
use libskrynia::fs::{self, RootKind, disk::DiskMode};

#[test]
fn qualified_paths_route_each_root_independently() {
    let (dir, k) = setup("k10-qualified-roots");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    for (root, path, value) in [
        (
            RootKind::User,
            b"user/k.bin".as_slice(),
            b"user-value".as_slice(),
        ),
        (RootKind::Operator, b"operator/k.bin", b"operator-value"),
    ] {
        let fd = s
            .open_disk(Some(root), &mut fds, path, DiskMode::read_write(), 0)
            .unwrap();
        assert_eq!(s.write(&mut fds, fd, value).unwrap(), value.len());
        s.sync(&mut fds, fd).unwrap();
        s.close(&mut fds, fd).unwrap();
        assert!(s.exists(Some(root), path).unwrap());
    }
    assert_eq!(
        std::fs::read(dir.join("user/k.bin")).unwrap(),
        b"user-value"
    );
    assert_eq!(
        std::fs::read(dir.join("operator/k.bin")).unwrap(),
        b"operator-value"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

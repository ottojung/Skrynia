use super::support::setup;
use libskrynia::fs::{self, RootKind, disk::DiskMode};

#[test]
fn unqualified_paths_are_rejected_by_every_fs_entry_point() {
    let (_dir, k) = setup("k10-unqualified");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    let mode = fs::node::MkdirMode::create_if_absent();
    for p in [b"k.bin".as_slice(), b"", b"/user/k.bin", b"programs/k.bin"] {
        assert!(s.exists(None, p).is_err(), "{p:?}");
        assert!(s.is_dir(None, p).is_err(), "{p:?}");
        assert!(s.mkdir(None, p, mode).is_err(), "{p:?}");
        assert!(s.delete(None, p).is_err(), "{p:?}");
        assert!(s.rename(None, p, b"user/x").is_err(), "{p:?}");
        assert!(s.open_directory(None, &mut fds, p).is_err(), "{p:?}");
        for m in [DiskMode::read_only(), DiskMode::read_write()] {
            assert!(s.open_disk(None, &mut fds, p, m).is_err(), "{p:?}");
        }
    }
}

#[test]
fn no_implicit_fallback_between_confined_roots() {
    let (dir, k) = setup("k10-no-fallback");
    let s = k.fs();
    let mut fds = fs::table::Table::new();
    let fd = s
        .open_disk(None, &mut fds, b"user/only.bin", DiskMode::read_write())
        .unwrap();
    s.close(&mut fds, fd).unwrap();
    assert!(dir.join("user/only.bin").is_file());
    assert!(!dir.join("operator/only.bin").exists());
    assert!(
        !s.exists(Some(RootKind::Operator), b"operator/only.bin")
            .unwrap()
    );
    assert!(
        s.open_disk(None, &mut fds, b"operator/only.bin", DiskMode::read_only())
            .is_err()
    );
}

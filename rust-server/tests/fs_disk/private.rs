use std::{fs as stdfs, os::unix::fs::PermissionsExt};
#[test]
fn newly_created_disk_files_exclude_group_and_other() {
    let (dir, k) = super::setup::setup("skrynia-fs-private");
    let mut fds = libskrynia::fs::table::Table::new();
    let fd = k
        .fs()
        .open_disk(
            None,
            &mut fds,
            b"private",
            libskrynia::fs::disk::DiskMode::create_new_rw(),
            0,
        )
        .unwrap();
    k.fs().close(&mut fds, fd).unwrap();
    let mode = stdfs::metadata(dir.join("user/private"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o077, 0);
    stdfs::remove_dir_all(&dir).unwrap();
}

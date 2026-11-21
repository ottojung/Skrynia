use skrynia_server::fs;
use std::{fs as stdfs, os::unix::fs::PermissionsExt};
#[test]
fn newly_created_disk_files_exclude_group_and_other() {
    let (dir, k) = super::setup::setup("skrynia-fs-private");
    let mut fds = fs::table::Table::new();
    let fd = k
        .fs()
        .open_disk(
            None,
            &mut fds,
            b"private",
            fs::disk::DiskMode::create_new_rw(),
        )
        .unwrap();
    k.fs().close(&mut fds, fd).unwrap();
    let mode = stdfs::metadata(dir.join("data/private"))
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o077, 0);
    stdfs::remove_dir_all(&dir).unwrap();
}

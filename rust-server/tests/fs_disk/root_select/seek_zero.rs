use super::support::setup;
use libskrynia::fs::{self, disk::DiskMode};

#[test]
fn zero_relative_seek_preserves_position_after_shrink() {
    let (dir, k) = setup("abi31-seek-zero-shrink");
    let fs = k.fs();
    let mut fds = fs::table::Table::new();
    let ram = fs.open_ram(&mut fds).unwrap();
    fs.truncate_fd(&mut fds, ram, 16).unwrap();
    assert_eq!(fs.seek_rel(&mut fds, ram, 12), Ok(12));
    fs.truncate_fd(&mut fds, ram, 4).unwrap();
    assert_eq!(fs.seek_rel(&mut fds, ram, 0), Ok(12));
    assert!(fs.seek_rel(&mut fds, ram, -1).is_err());
    assert_eq!(fs.seek_rel(&mut fds, ram, -8), Ok(4));
    let disk = fs
        .open_disk(None, &mut fds, b"user/file", DiskMode::read_write())
        .unwrap();
    fs.truncate_fd(&mut fds, disk, 16).unwrap();
    assert_eq!(fs.seek_rel(&mut fds, disk, 12), Ok(12));
    fs.truncate_fd(&mut fds, disk, 4).unwrap();
    assert_eq!(fs.seek_rel(&mut fds, disk, 0), Ok(12));
    assert!(fs.seek_rel(&mut fds, disk, -1).is_err());
    assert_eq!(fs.seek_rel(&mut fds, disk, -8), Ok(4));
    std::fs::remove_dir_all(dir).unwrap();
}

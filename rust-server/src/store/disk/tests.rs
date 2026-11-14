use super::{clean_write_error, write_atomic, write_errno};
use std::{fs, io};
#[test]
fn failures_before_install_leave_destination_unmodified() {
    let root = std::env::temp_dir().join(format!("skrynia-atomic-fail-{}", crate::random_id()));
    fs::create_dir_all(root.join("record")).unwrap();
    let error = write_atomic(&root.join("record"), b"new", true).unwrap_err();
    assert!(clean_write_error(&error));
    assert!(root.join("record").is_dir());
    fs::remove_dir_all(root).unwrap();
}
#[test]
fn atomic_failure_preserves_disk_full_errno_and_commit_boundary() {
    let failed = super::failure::annotate(io::Error::from_raw_os_error(libc::ENOSPC), false);
    assert!(clean_write_error(&failed));
    assert_eq!(write_errno(&failed), Some(libc::ENOSPC));
    assert_eq!(crate::error::transient_io(&failed).0.as_u16(), 507);
    let installed = super::failure::annotate(io::Error::from_raw_os_error(libc::ENOSPC), true);
    assert!(!clean_write_error(&installed));
}

use super::{DiskMode, Entry};
use std::os::fd::FromRawFd;

pub fn open_disk(
    root: &crate::fs::root::Root,
    rel: &std::path::Path,
    mode: DiskMode,
) -> Result<Entry, &'static str> {
    let mut flags = libc::O_CLOEXEC | libc::O_NOFOLLOW;
    if mode.read && mode.write {
        flags |= libc::O_RDWR;
    } else if mode.write {
        flags |= libc::O_WRONLY;
    } else {
        flags |= libc::O_RDONLY;
    }
    if mode.create {
        flags |= libc::O_CREAT;
    }
    if mode.exclusive {
        flags |= libc::O_EXCL;
    }
    if mode.truncate {
        flags |= libc::O_TRUNC;
    }
    let fd = super::super::root::rel::open_rel(root.fd(), rel, flags, 0o666)?;
    let file = unsafe { std::fs::File::from_raw_fd(std::os::fd::IntoRawFd::into_raw_fd(fd)) };
    Ok(Entry {
        file,
        can_read: mode.read,
        can_write: mode.write,
    })
}

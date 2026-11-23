use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

pub fn open_path(root: i32, parts: &[std::ffi::CString]) -> Result<OwnedFd, &'static str> {
    if parts.is_empty() {
        return Err("empty");
    }
    let mut dirfd = root;
    let mut cur = None;
    for (i, name) in parts.iter().enumerate() {
        let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
        let fd = unsafe { libc::openat(dirfd, name.as_ptr(), flags) };
        if fd < 0 {
            let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
            return Err(match e {
                libc::ENOENT => "missing",
                libc::EACCES | libc::EPERM => "perm",
                _ => "open",
            });
        }
        let owned = unsafe { OwnedFd::from_raw_fd(fd) };
        if i + 1 == parts.len() {
            return Ok(owned);
        }
        dirfd = owned.as_raw_fd();
        cur = Some(owned);
    }
    cur.ok_or("empty")
}

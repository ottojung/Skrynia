use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};

fn stat_fd(fd: std::os::fd::RawFd) -> Result<(libc::dev_t, libc::ino_t), String> {
    let mut st = std::mem::MaybeUninit::<libc::stat>::uninit();
    let rc = unsafe { libc::fstat(fd, st.as_mut_ptr()) };
    if rc != 0 {
        return Err(format!("stat: {}", std::io::Error::last_os_error()));
    }
    let st = unsafe { st.assume_init() };
    Ok((st.st_dev, st.st_ino))
}

pub fn is_ancestor(
    ancestor: &crate::fs::root::Root,
    child: &crate::fs::root::Root,
) -> Result<bool, String> {
    let aid = ancestor.id();
    if aid == child.id() {
        return Ok(true);
    }
    let dup = unsafe { libc::fcntl(child.fd(), libc::F_DUPFD_CLOEXEC, 0) };
    if dup < 0 {
        return Err(format!("dup: {}", std::io::Error::last_os_error()));
    }
    let mut cur = unsafe { OwnedFd::from_raw_fd(dup) };
    for _ in 0..4096 {
        let id = stat_fd(cur.as_raw_fd())?;
        if id == aid {
            return Ok(true);
        }
        let pfd = unsafe {
            libc::openat(
                cur.as_raw_fd(),
                c"..".as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            )
        };
        if pfd < 0 {
            return Err(format!("open parent: {}", std::io::Error::last_os_error()));
        }
        let parent = unsafe { OwnedFd::from_raw_fd(pfd) };
        if stat_fd(parent.as_raw_fd())? == id {
            return Ok(false);
        }
        cur = parent;
    }
    Err("root ancestry walk exceeded limit".into())
}

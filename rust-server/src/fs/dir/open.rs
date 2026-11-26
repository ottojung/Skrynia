use super::Entry;

pub fn open_dir(
    root: &crate::fs::root::Root,
    rel: &std::path::Path,
) -> Result<Entry, &'static str> {
    let parts = super::super::root::walk::parts(rel)?;
    let fd = super::path::open_path(root.fd(), &parts)?;
    entry(fd)
}

// "." opens the selected authorized root itself, never a parent path.
pub fn open_root(root: &crate::fs::root::Root) -> Result<Entry, &'static str> {
    use std::os::fd::FromRawFd;
    let name = std::ffi::CString::new(".").map_err(|_| "invalid")?;
    let fd = unsafe {
        libc::openat(
            root.fd(),
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        return Err("open");
    }
    entry(unsafe { std::os::fd::OwnedFd::from_raw_fd(fd) })
}

fn entry(fd: std::os::fd::OwnedFd) -> Result<Entry, &'static str> {
    let stream = super::stream::Stream::open(&fd)?;
    Ok(Entry {
        stream,
        pending: None,
        fd,
    })
}

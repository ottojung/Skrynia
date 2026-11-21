use super::walk;
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};

pub fn open_rel(
    root: RawFd,
    rel: &std::path::Path,
    flags: i32,
    mode: u32,
) -> Result<OwnedFd, &'static str> {
    let parts = walk::parts(rel)?;
    if parts.len() == 1 {
        return open_final(root, &parts[0], flags, mode);
    }
    let mut cur = walk::open_dir_at(root, &parts[0])?;
    for name in &parts[1..parts.len() - 1] {
        let next = walk::open_dir_at(cur.as_raw_fd(), name)?;
        cur = next;
    }
    open_final(cur.as_raw_fd(), parts.last().unwrap(), flags, mode)
}

fn open_final(dir: RawFd, name: &CString, flags: i32, mode: u32) -> Result<OwnedFd, &'static str> {
    let fd = unsafe {
        libc::openat(
            dir,
            name.as_ptr(),
            flags | libc::O_CLOEXEC,
            mode as libc::c_uint,
        )
    };
    if fd < 0 {
        let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        return Err(match e {
            libc::EEXIST => "exists",
            libc::ENOENT => "missing",
            libc::ENOSPC => "nospace",
            libc::EACCES | libc::EPERM => "perm",
            _ => "open",
        });
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

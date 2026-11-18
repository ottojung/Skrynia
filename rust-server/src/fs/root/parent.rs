use super::walk;
use std::ffi::CString;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd, RawFd};

pub struct Parent {
    pub dir: OwnedFd,
    pub name: CString,
}

pub fn parent(root: RawFd, rel: &std::path::Path) -> Result<Parent, &'static str> {
    let parts = walk::parts(rel)?;
    if parts.len() == 1 {
        let dup = unsafe { libc::fcntl(root, libc::F_DUPFD_CLOEXEC, 0) };
        if dup < 0 {
            return Err("open");
        }
        let dir = unsafe { OwnedFd::from_raw_fd(dup) };
        return Ok(Parent {
            dir,
            name: parts.into_iter().next().unwrap(),
        });
    }
    let mut cur = walk::open_dir_at(root, &parts[0])?;
    for name in &parts[1..parts.len() - 1] {
        cur = walk::open_dir_at(cur.as_raw_fd(), name)?;
    }
    Ok(Parent {
        dir: cur,
        name: parts.into_iter().last().unwrap(),
    })
}

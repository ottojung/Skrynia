use std::ffi::CStr;
use std::os::fd::{AsRawFd, OwnedFd};

pub fn read_names(fd: &OwnedFd) -> Result<Vec<String>, &'static str> {
    let dup = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) };
    if dup < 0 {
        return Err("open");
    }
    let dir = unsafe { libc::fdopendir(dup) };
    if dir.is_null() {
        unsafe { libc::close(dup) };
        return Err("open");
    }
    let mut out = Vec::new();
    loop {
        let ent = unsafe { libc::readdir(dir) };
        if ent.is_null() {
            break;
        }
        let name = unsafe { CStr::from_ptr((*ent).d_name.as_ptr()) };
        let s = name.to_string_lossy();
        if s != "." && s != ".." {
            out.push(s.into_owned());
        }
    }
    unsafe { libc::closedir(dir) };
    Ok(out)
}

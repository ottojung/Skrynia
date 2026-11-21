use std::ffi::CStr;
use std::os::fd::{AsRawFd, OwnedFd};

pub struct Stream(*mut libc::DIR);
impl Stream {
    pub fn open(fd: &OwnedFd) -> Result<Self, &'static str> {
        let dup = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_DUPFD_CLOEXEC, 0) };
        if dup < 0 {
            return Err("open");
        }
        let dir = unsafe { libc::fdopendir(dup) };
        if dir.is_null() {
            unsafe { libc::close(dup) };
            return Err("open");
        }
        Ok(Self(dir))
    }
    pub fn next(&mut self) -> Result<Option<Vec<u8>>, &'static str> {
        loop {
            let ent = unsafe {
                *libc::__errno_location() = 0;
                libc::readdir(self.0)
            };
            if ent.is_null() {
                let errno = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
                return if errno == 0 { Ok(None) } else { Err("io") };
            }
            let name = unsafe { CStr::from_ptr((*ent).d_name.as_ptr()) };
            let s = name.to_str().map_err(|_| "invalid")?;
            if s != "." && s != ".." {
                return Ok(Some(s.as_bytes().to_vec()));
            }
        }
    }
}
impl Drop for Stream {
    fn drop(&mut self) {
        unsafe { libc::closedir(self.0) };
    }
}

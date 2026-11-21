use std::ffi::CString;
use std::os::raw::{c_char, c_int};

unsafe extern "C" {
    fn open(path: *const c_char, oflag: c_int) -> c_int;
    fn close(fd: c_int) -> c_int;
    fn fcntl(fd: c_int, cmd: c_int, arg: c_int) -> c_int;
}

const O_RDONLY: c_int = 0;
const F_SETFD: c_int = 2;

pub struct InheritedFd(c_int);

impl InheritedFd {
    pub fn open_secret() -> Self {
        let path = CString::new("/dev/null").unwrap();
        let fd = unsafe { open(path.as_ptr(), O_RDONLY) };
        assert!(fd >= 0, "open /dev/null failed");
        let rc = unsafe { fcntl(fd, F_SETFD, 0) };
        assert!(rc == 0, "fcntl(F_SETFD, 0) failed");
        Self(fd)
    }
}

impl Drop for InheritedFd {
    fn drop(&mut self) {
        unsafe {
            let _ = close(self.0);
        }
    }
}

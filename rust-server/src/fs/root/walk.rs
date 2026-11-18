use std::ffi::CString;
use std::os::fd::{FromRawFd, OwnedFd, RawFd};
use std::path::{Component, Path};

pub fn parts(path: &Path) -> Result<Vec<CString>, &'static str> {
    let mut out = Vec::new();
    for c in path.components() {
        match c {
            Component::Normal(s) => {
                let b = s.as_encoded_bytes();
                if b.contains(&0) {
                    return Err("null");
                }
                out.push(CString::new(b).map_err(|_| "null")?);
            }
            Component::CurDir => {}
            _ => return Err("escape"),
        }
    }
    if out.is_empty() {
        return Err("empty");
    }
    Ok(out)
}

pub fn open_dir_at(dir: RawFd, name: &CString) -> Result<OwnedFd, &'static str> {
    let fd = unsafe {
        libc::openat(
            dir,
            name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC,
        )
    };
    if fd < 0 {
        let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        return Err(if e == libc::ENOENT { "missing" } else { "open" });
    }
    Ok(unsafe { OwnedFd::from_raw_fd(fd) })
}

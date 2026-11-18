use std::os::fd::AsRawFd;

#[derive(Clone, Copy, Debug)]
pub struct MkdirMode {
    pub create: bool,
    pub exclusive: bool,
}

impl MkdirMode {
    pub fn create_if_absent() -> Self {
        Self {
            create: true,
            exclusive: false,
        }
    }
    pub fn create_new() -> Self {
        Self {
            create: true,
            exclusive: true,
        }
    }
}

pub fn mkdir(
    root: &crate::fs::root::Root,
    rel: &std::path::Path,
    mode: MkdirMode,
) -> Result<(), &'static str> {
    let p = super::super::root::parent::parent(root.fd(), rel)?;
    if !mode.create {
        return Err("mode");
    }
    let rc = unsafe { libc::mkdirat(p.dir.as_raw_fd(), p.name.as_ptr(), 0o777) };
    if rc == 0 {
        return Ok(());
    }
    let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    if e == libc::EEXIST && !mode.exclusive {
        let mut st: libc::stat = unsafe { std::mem::zeroed() };
        let r = unsafe {
            libc::fstatat(
                p.dir.as_raw_fd(),
                p.name.as_ptr(),
                &mut st,
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if r == 0 {
            return Ok(());
        }
    }
    Err("mkdir")
}

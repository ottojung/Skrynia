use std::os::fd::AsRawFd;

pub fn rename(
    root: &crate::fs::root::Root,
    old: &std::path::Path,
    new: &std::path::Path,
) -> Result<(), &'static str> {
    let o = super::super::root::parent::parent(root.fd(), old)?;
    let n = super::super::root::parent::parent(root.fd(), new)?;
    let rc = unsafe {
        libc::renameat(
            o.dir.as_raw_fd(),
            o.name.as_ptr(),
            n.dir.as_raw_fd(),
            n.name.as_ptr(),
        )
    };
    if rc == 0 {
        return Ok(());
    }
    let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    Err(match e {
        libc::EXDEV => "unsupported",
        libc::ENOENT => "missing",
        _ => "rename",
    })
}

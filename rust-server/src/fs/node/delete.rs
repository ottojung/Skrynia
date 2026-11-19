use std::os::fd::AsRawFd;

pub fn delete(root: &crate::fs::root::Root, rel: &std::path::Path) -> Result<(), &'static str> {
    let p = match super::super::root::parent::parent(root.fd(), rel) {
        Ok(p) => p,
        Err("missing") => return Ok(()),
        Err(e) => return Err(e),
    };
    let rc = unsafe { libc::unlinkat(p.dir.as_raw_fd(), p.name.as_ptr(), 0) };
    if rc == 0 {
        return Ok(());
    }
    let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
    if e == libc::ENOENT {
        return Ok(());
    }
    if e == libc::EISDIR {
        let rc = unsafe { libc::unlinkat(p.dir.as_raw_fd(), p.name.as_ptr(), libc::AT_REMOVEDIR) };
        if rc == 0 {
            return Ok(());
        }
        let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        if e == libc::ENOENT {
            return Ok(());
        }
    }
    Err("delete")
}

use std::os::fd::AsRawFd;

fn stat(root: &crate::fs::root::Root, rel: &std::path::Path) -> Result<libc::stat, &'static str> {
    let p = super::super::root::parent::parent(root.fd(), rel)?;
    let mut st: libc::stat = unsafe { std::mem::zeroed() };
    let rc = unsafe {
        libc::fstatat(
            p.dir.as_raw_fd(),
            p.name.as_ptr(),
            &mut st,
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if rc < 0 {
        let e = std::io::Error::last_os_error().raw_os_error().unwrap_or(0);
        return Err(if e == libc::ENOENT { "missing" } else { "stat" });
    }
    Ok(st)
}

pub fn exists(root: &crate::fs::root::Root, rel: &std::path::Path) -> Result<bool, &'static str> {
    match stat(root, rel) {
        Ok(_) => Ok(true),
        Err("missing") => Ok(false),
        Err(e) => Err(e),
    }
}

pub fn is_dir(root: &crate::fs::root::Root, rel: &std::path::Path) -> Result<bool, &'static str> {
    match stat(root, rel) {
        Ok(st) => Ok((st.st_mode & libc::S_IFMT) == libc::S_IFDIR),
        Err("missing") => Ok(false),
        Err(e) => Err(e),
    }
}

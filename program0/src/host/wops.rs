use super::root_abi as sys;

pub fn sync_dir(path: &str) -> Result<(), i32> {
    let fd = unsafe { sys::open_directory(path.as_ptr() as u32, path.len() as u32) };
    if fd < 0 {
        return Err(fd);
    }
    let synced = unsafe { sys::sync(fd) };
    let closed = unsafe { sys::close(fd) };
    if synced < 0 {
        Err(synced)
    } else if closed < 0 {
        Err(closed)
    } else {
        Ok(())
    }
}

pub fn write_all(fd: i32, bytes: &[u8]) -> i32 {
    let mut written = 0;
    while written < bytes.len() {
        let remaining = &bytes[written..];
        let len = remaining.len().min(65536);
        let n = unsafe { sys::write(fd, remaining.as_ptr() as u32, len as u32) };
        if n <= 0 {
            return if n < 0 { n } else { -5 };
        }
        written += n as usize;
    }
    0
}

pub fn sync_and_close(fd: i32, status: i32) -> i32 {
    let synced = if status == 0 {
        unsafe { sys::sync(fd) }
    } else {
        status
    };
    let closed = unsafe { sys::close(fd) };
    if synced < 0 { synced } else { closed }
}

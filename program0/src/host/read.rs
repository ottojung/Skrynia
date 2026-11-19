use super::root_abi as sys;
const MAX: usize = 16 * 1024 * 1024;

pub fn read_stream(fd: i32, max: usize) -> Result<Vec<u8>, i32> {
    let mut out = Vec::new();
    loop {
        let mut buf = [0u8; 8192];
        let n = unsafe { sys::read(fd, buf.as_mut_ptr() as u32, buf.len() as u32) };
        if n < 0 {
            return Err(n);
        }
        if n == 0 {
            return Ok(out);
        }
        let n = n as usize;
        if n > max.saturating_sub(out.len()) {
            return Err(-10);
        }
        out.extend_from_slice(&buf[..n]);
    }
}

pub fn read_file(path: &str) -> Result<Option<Vec<u8>>, i32> {
    let ptr = path.as_ptr() as u32;
    let len = path.len() as u32;
    let exists = unsafe { sys::exists(ptr, len) };
    if exists < 0 {
        return Err(exists);
    }
    if exists == 0 {
        return Ok(None);
    }
    let fd = unsafe { sys::open_disk(ptr, len, 1) };
    if fd < 0 {
        return Err(fd);
    }
    let result = read_stream(fd, MAX).map(Some);
    let closed = unsafe { sys::close(fd) };
    if closed < 0 {
        return Err(closed);
    }
    result
}

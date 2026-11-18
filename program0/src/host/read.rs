use super::root_abi as sys;
const MAX: usize = 16 * 1024 * 1024;

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
    let mut out = Vec::new();
    let result = loop {
        let mut buf = [0u8; 8192];
        let n = unsafe { sys::read(fd, buf.as_mut_ptr() as u32, buf.len() as u32) };
        if n < 0 {
            break Err(n);
        }
        if n == 0 {
            break Ok(Some(out));
        }
        if n as usize > MAX - out.len() {
            break Err(-10);
        }
        out.extend_from_slice(&buf[..n as usize]);
    };
    let closed = unsafe { sys::close(fd) };
    if closed < 0 {
        return Err(closed);
    }
    result
}

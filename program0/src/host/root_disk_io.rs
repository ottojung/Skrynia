#[cfg(target_arch = "wasm32")]
use crate::root_abi as host;

#[cfg(target_arch = "wasm32")]
pub fn read_all(path: &str) -> Result<Option<Vec<u8>>, i32> {
    let fd = unsafe { host::open_disk(path.as_ptr() as u32, path.len() as u32, 1) };
    if fd == -2 {
        return Ok(None);
    }
    if fd < 0 {
        return Err(fd);
    }
    let mut out = Vec::new();
    loop {
        let mut b = [0u8; 8192];
        let n = unsafe { host::read(fd, b.as_mut_ptr() as u32, b.len() as u32) };
        if n <= 0 {
            unsafe { host::close(fd) };
            if n < 0 {
                return Err(n);
            }
            return Ok(Some(out));
        }
        out.extend_from_slice(&b[..n as usize]);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn write_bytes(fd: i32, bytes: &[u8]) -> Result<(), i32> {
    for b in bytes.chunks(8192) {
        let mut done = 0;
        while done < b.len() {
            let rest = &b[done..];
            let n = unsafe { host::write(fd, rest.as_ptr() as u32, rest.len() as u32) };
            if n <= 0 {
                return Err(if n == 0 { -5 } else { n });
            }
            done += n as usize;
        }
    }
    let rc = unsafe { host::sync(fd) };
    if rc < 0 { Err(rc) } else { Ok(()) }
}

#[cfg(target_arch = "wasm32")]
use crate::root_abi as host;

#[cfg(target_arch = "wasm32")]
pub fn ensure(dir: &str) -> Result<(), i32> {
    let n = unsafe { host::mkdir(dir.as_ptr() as u32, dir.len() as u32, 4) };
    if n < 0 { Err(n) } else { Ok(()) }
}

#[cfg(target_arch = "wasm32")]
pub fn list(dir: &str) -> Result<Vec<String>, i32> {
    let fd = unsafe { host::open_directory(dir.as_ptr() as u32, dir.len() as u32) };
    if fd == -2 {
        return Ok(Vec::new());
    }
    if fd < 0 {
        return Err(fd);
    }
    let mut names = Vec::new();
    loop {
        let mut b = [0u8; 256];
        let n = unsafe { host::next_entry(fd, b.as_mut_ptr() as u32, b.len() as u32) };
        if n <= 0 {
            unsafe { host::close(fd) };
            if n < 0 {
                return Err(n);
            }
            return Ok(names);
        }
        let s = String::from_utf8(b[..n as usize].to_vec()).map_err(|_| -1)?;
        names.push(s);
    }
}

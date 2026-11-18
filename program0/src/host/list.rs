use super::root_abi as sys;

pub fn list_dir(path: &str) -> Result<Vec<String>, i32> {
    let p = path.as_ptr() as u32;
    let l = path.len() as u32;
    let exists = unsafe { sys::exists(p, l) };
    if exists < 0 {
        return Err(exists);
    }
    if exists == 0 {
        return Ok(Vec::new());
    }
    let fd = unsafe { sys::open_directory(p, l) };
    if fd < 0 {
        return Err(fd);
    }
    let mut out = Vec::new();
    let result = loop {
        let mut buf = [0u8; 512];
        let n = unsafe { sys::next_entry(fd, buf.as_mut_ptr() as u32, buf.len() as u32) };
        if n < 0 {
            break Err(n);
        }
        if n == 0 {
            break Ok(out);
        }
        match std::str::from_utf8(&buf[..n as usize]) {
            Ok(name) => out.push(name.to_string()),
            Err(_) => break Err(-1),
        }
    };
    let closed = unsafe { sys::close(fd) };
    if closed < 0 {
        return Err(closed);
    }
    result
}

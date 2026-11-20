pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "read",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             fd: i32,
             dst: u32,
             len: u32|
             -> i32 {
                if len > crate::boot::limits::MAX_IO {
                    return crate::boot::vm::codes::ERR_LIMIT;
                }
                if let Err(e) = super::guard::check(c.data(), crate::fs::Fd(fd)) {
                    return e;
                }
                let m = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                if dst as u64 + len as u64 > m.data_size(&c) as u64 {
                    return crate::boot::vm::codes::ERR_RANGE;
                }
                let mut buf = vec![0u8; len.min(crate::boot::limits::IO_CHUNK) as usize];
                let fs = c.data().fs.clone();
                let n = match fs.read(&mut c.data_mut().fds, crate::fs::Fd(fd), &mut buf) {
                    Ok(n) => n,
                    Err(e) => return crate::boot::vm::err::map(e),
                };
                let m = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                if crate::boot::vm::mem::write_buf(&mut c, m, dst, &buf[..n]).is_err() {
                    return crate::boot::vm::codes::ERR_RANGE;
                }
                n as i32
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

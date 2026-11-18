pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "write",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             fd: i32,
             src: u32,
             len: u32|
             -> i32 {
                if len > crate::boot::limits::MAX_IO {
                    return crate::boot::vm::codes::ERR_LIMIT;
                }
                let mem = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let buf = match crate::boot::vm::mem::read_buf(&mut c, mem, src, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let fs = c.data().fs.clone();
                match fs.write(&mut c.data_mut().fds, crate::fs::Fd(fd), &buf) {
                    Ok(n) => n as i32,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

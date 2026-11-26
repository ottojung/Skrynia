// Transitional derived bulk write built from byte writes; remove after test port.
type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "write",
            |mut c: wasmtime::Caller<'_, Vm>, fd: i32, src: u32, len: u32| -> i32 {
                if len > crate::boot::limits::MAX_IO {
                    return crate::boot::vm::codes::ERR_LIMIT;
                }
                if let Err(e) = super::guard::check(c.data(), crate::fs::Fd(fd)) {
                    return e;
                }
                let mem = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                if src as u64 + len as u64 > mem.data_size(&c) as u64 {
                    return crate::boot::vm::codes::ERR_RANGE;
                }
                let chunk = len.min(crate::boot::limits::IO_CHUNK);
                let buf = match crate::boot::vm::mem::read_buf(&mut c, mem, src, chunk) {
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

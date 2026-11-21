fn mode(bits: u32) -> crate::fs::disk::DiskMode {
    crate::fs::disk::DiskMode {
        read: bits & 1 != 0,
        write: bits & 2 != 0,
        create: bits & 4 != 0,
        exclusive: bits & 8 != 0,
        truncate: bits & 16 != 0,
    }
}
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "open_disk",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             ptr: u32,
             len: u32,
             m: u32|
             -> i32 {
                let mem = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let path = match crate::boot::vm::mem::read_buf(&mut c, mem, ptr, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let fs = c.data().fs.clone();
                match fs.open_disk(&mut c.data_mut().fds, &path, mode(m)) {
                    Ok(fd) => fd.0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

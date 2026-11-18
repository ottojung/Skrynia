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
                let m = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let buf = match crate::boot::vm::mem::read_buf(&mut c, m, src, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                match c.data().fs.write(crate::fs::Fd(fd), &buf) {
                    Ok(n) => n as i32,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

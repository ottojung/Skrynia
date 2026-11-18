pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "seek",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32, off: u64| -> i32 {
                match c.data().fs.seek(crate::fs::Fd(fd), off) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "truncate",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32, size: u64| -> i32 {
                match c.data().fs.truncate_fd(crate::fs::Fd(fd), size) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "close",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32| -> i32 {
                match c.data().fs.close(crate::fs::Fd(fd)) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "sync",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32| -> i32 {
                match c.data().fs.sync(crate::fs::Fd(fd)) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

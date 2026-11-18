pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "close",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32| -> i32 {
                let fs = c.data().fs.clone();
                match fs.close(&mut c.data_mut().fds, crate::fs::Fd(fd)) {
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
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32| -> i32 {
                let fs = c.data().fs.clone();
                match fs.sync(&mut c.data_mut().fds, crate::fs::Fd(fd)) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

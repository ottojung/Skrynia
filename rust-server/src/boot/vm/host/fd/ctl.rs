pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "close",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32| -> i32 {
                let f = crate::fs::Fd(fd);
                let fs = c.data().fs.clone();
                match fs.close(&mut c.data_mut().fds, f) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::host::memory::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia_v3_1",
            "sync",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32| -> i32 {
                let f = crate::fs::Fd(fd);
                let fs = c.data().fs.clone();
                match fs.sync(&mut c.data_mut().fds, f) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::host::memory::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "seek",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32, off: u64| -> i32 {
                let f = crate::fs::Fd(fd);
                if let Err(e) = super::guard::check(c.data(), f) {
                    return e;
                }
                let fs = c.data().fs.clone();
                match fs.seek(&mut c.data_mut().fds, f, off) {
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
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32, size: u64| -> i32 {
                let f = crate::fs::Fd(fd);
                if let Err(e) = super::guard::check(c.data(), f) {
                    return e;
                }
                let fs = c.data().fs.clone();
                match fs.truncate_fd(&mut c.data_mut().fds, f, size) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

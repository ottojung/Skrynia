pub fn add_close(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "close",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, fd: i32| -> i32 {
                let f = crate::fs::Fd(fd);
                if let Err(e) = super::guard::check(c.data(), f) {
                    return e;
                }
                let fs = c.data().fs.clone();
                match fs.close(&mut c.data_mut().fds, f) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

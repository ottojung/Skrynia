type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "seek",
            |mut c: wasmtime::Caller<'_, Vm>, fd: i32, shift: i64| -> i64 {
                let fs = c.data().fs.clone();
                match fs.seek_rel(&mut c.data_mut().fds, crate::fs::Fd(fd), shift) {
                    Ok(pos) => i64::try_from(pos)
                        .unwrap_or(i64::from(crate::boot::vm::host::memory::E_RANGE)),
                    Err("badfd") => i64::from(crate::boot::vm::host::memory::E_BADFD),
                    Err(e) => i64::from(crate::boot::vm::host::memory::map_fs(e)),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia_v3_1",
            "truncate",
            |mut c: wasmtime::Caller<'_, Vm>, fd: i32, size: u64| -> i32 {
                let fs = c.data().fs.clone();
                match fs.truncate_fd(&mut c.data_mut().fds, crate::fs::Fd(fd), size) {
                    Ok(()) => 0,
                    Err("badfd") => crate::boot::vm::host::memory::E_BADFD,
                    Err(e) => crate::boot::vm::host::memory::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

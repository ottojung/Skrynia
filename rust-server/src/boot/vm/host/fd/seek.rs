type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "seek",
            |mut c: wasmtime::Caller<'_, Vm>, fd: i32, shift: i64| -> i64 {
                if super::guard::check(c.data(), crate::fs::Fd(fd)).is_err() {
                    return i64::from(crate::boot::vm::host::ram::args::E_BADFD);
                }
                let fs = c.data().fs.clone();
                match fs.seek_rel(&mut c.data_mut().fds, crate::fs::Fd(fd), shift) {
                    Ok(pos) => pos as i64,
                    Err("badfd") => i64::from(crate::boot::vm::host::ram::args::E_BADFD),
                    Err(e) => i64::from(crate::boot::vm::host::ram::args::map_fs(e)),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "truncate",
            |mut c: wasmtime::Caller<'_, Vm>, fd: i32, size: u64| -> i32 {
                if super::guard::check(c.data(), crate::fs::Fd(fd)).is_err() {
                    return crate::boot::vm::host::ram::args::E_BADFD;
                }
                let fs = c.data().fs.clone();
                match fs.truncate_fd(&mut c.data_mut().fds, crate::fs::Fd(fd), size) {
                    Ok(()) => 0,
                    Err("badfd") => crate::boot::vm::host::ram::args::E_BADFD,
                    Err(e) => crate::boot::vm::host::ram::args::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

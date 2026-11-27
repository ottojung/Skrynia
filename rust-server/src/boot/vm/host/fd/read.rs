type Vm = crate::boot::vm::state::Vm;
type Caller<'a> = wasmtime::Caller<'a, Vm>;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "read1",
            |mut c: Caller<'_>, fd: i32| -> i32 {
                let fs = c.data().fs.clone();
                match fs.read1(&mut c.data_mut().fds, crate::fs::Fd(fd)) {
                    Ok(Some(b)) => i32::from(b),
                    Ok(None) => -1,
                    Err("badfd") => crate::boot::vm::host::memory::E_BADFD,
                    Err(e) => crate::boot::vm::host::memory::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

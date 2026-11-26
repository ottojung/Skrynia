type Vm = crate::boot::vm::state::Vm;
type Caller<'a> = wasmtime::Caller<'a, Vm>;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "write1",
            |mut c: Caller<'_>, fd: i32, byte: u32| -> i32 {
                if byte > 255 {
                    return crate::boot::vm::host::ram::args::E_INVAL;
                }
                let fs = c.data().fs.clone();
                match fs.write1(&mut c.data_mut().fds, crate::fs::Fd(fd), byte as u8) {
                    Ok(()) => 0,
                    Err("badfd") => crate::boot::vm::host::ram::args::E_BADFD,
                    Err(e) => crate::boot::vm::host::ram::args::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

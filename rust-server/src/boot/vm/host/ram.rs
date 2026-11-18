pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "open_ram",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>| -> i32 {
                c.data().fs.open_ram().0
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "cas_ram_u64",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             _fd: i32,
             off: u64,
             exp: u64,
             des: u64|
             -> i64 {
                match c.data().ram.cas_u64(off as usize, exp, des) {
                    Ok(old) => old as i64,
                    Err(_) => crate::boot::vm::codes::ERR_RANGE as i64,
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

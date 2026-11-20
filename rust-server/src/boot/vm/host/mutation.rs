pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "try_mutation_lock",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>| -> i32 {
                i32::from(c.data_mut().try_mutation_lock())
            },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "release_mutation_lock",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>| -> i32 {
                if c.data_mut().release_mutation_lock() {
                    0
                } else {
                    crate::boot::vm::codes::ERR_PERM
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

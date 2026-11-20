pub fn new_linker(
    engine: &wasmtime::Engine,
    scoped: bool,
) -> Result<wasmtime::Linker<crate::boot::vm::state::Vm>, i32> {
    let mut linker = wasmtime::Linker::new(engine);
    crate::boot::vm::host::fd::add(&mut linker).map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    if scoped {
        crate::boot::vm::host::pathops::add(&mut linker)
            .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
        crate::boot::vm::host::dir::add(&mut linker)
            .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
        crate::boot::vm::host::mutation::add(&mut linker)
            .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
        crate::boot::vm::host::sys::add(&mut linker)
            .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
        crate::boot::vm::host::eval::add(&mut linker)
            .map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    }
    Ok(linker)
}

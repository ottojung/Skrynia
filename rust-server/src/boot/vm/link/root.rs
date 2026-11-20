pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    crate::boot::vm::host::sys::add(linker)?;
    crate::boot::vm::host::ram::add(linker)?;
    crate::boot::vm::host::mutation::add(linker)?;
    crate::boot::vm::host::fd::add(linker)?;
    crate::boot::vm::host::pathops::add(linker)?;
    crate::boot::vm::host::dir::add(linker)?;
    crate::boot::vm::host::eval::add(linker)?;
    crate::boot::vm::host::program1::add(linker)?;
    Ok(())
}

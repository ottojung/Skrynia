mod next;
mod open;

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    open::add(linker)?;
    next::add(linker)?;
    Ok(())
}

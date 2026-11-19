mod ctl;
mod open;
mod read;
mod seek;
mod write;
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    open::add(linker)?;
    read::add(linker)?;
    write::add(linker)?;
    ctl::add(linker)?;
    seek::add(linker)?;
    Ok(())
}

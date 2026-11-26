mod ctl;
pub mod ctl_pub;
pub(crate) mod guard;
mod open;
pub mod read;
mod seek;
pub mod write;

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    open::add(linker)?;
    read::add(linker)?;
    write::add(linker)?;
    ctl::add(linker)?;
    seek::add(linker)?;
    Ok(())
}
pub fn add_stream(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    read::add(linker)?;
    write::add(linker)?;
    ctl_pub::add_close(linker)?;
    Ok(())
}

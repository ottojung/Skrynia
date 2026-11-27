mod bulk;
mod ctl;
mod open;
pub mod read;
mod seek;
pub mod write;

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    bulk::add(linker)?;
    open::add(linker)?;
    read::add(linker)?;
    write::add(linker)?;
    ctl::add(linker)?;
    seek::add(linker)?;
    Ok(())
}

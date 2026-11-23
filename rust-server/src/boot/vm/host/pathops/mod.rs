mod del;
mod ex;
mod mkdir;
mod rn;

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    mkdir::add(linker)?;
    del::add(linker)?;
    ex::add(linker)?;
    rn::add(linker)?;
    Ok(())
}

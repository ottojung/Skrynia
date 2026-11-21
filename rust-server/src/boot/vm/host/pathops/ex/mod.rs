mod exists;
mod isdir;
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    exists::add(linker)?;
    isdir::add(linker)
}

mod buffered;
mod cb;
mod check;
mod comp;
mod guest;
mod reg;
mod run;
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    add_funcs(linker)
}
pub fn add_funcs(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    reg::add(linker)?;
    buffered::add(linker)?;
    crate::boot::vm::host::fsscope::add(linker)
}

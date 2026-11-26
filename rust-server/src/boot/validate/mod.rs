mod exports;
mod imports;
mod shape;
mod signatures;
mod syscall;

pub fn check(module: &wasmtime::Module) -> Result<(), String> {
    exports::check(module)?;
    imports::check(module)?;
    Ok(())
}

/// Typed custom imports are matched against discovered signatures before use.
pub fn check_syscalls(
    module: &wasmtime::Module,
    signatures: &std::collections::BTreeMap<String, crate::Signature>,
) -> Result<(), String> {
    syscall::check(module, signatures)
}

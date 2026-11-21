mod apply;
mod check;
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "fs_delegate_scope",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             root: i32,
             path_ptr: u32,
             path_len: u32,
             ops: u32| { apply::scope(c, root, path_ptr, path_len, ops) },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

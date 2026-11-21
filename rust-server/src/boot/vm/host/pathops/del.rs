pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "delete",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, ptr: u32, len: u32| -> i32 {
                let mem = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let p = match crate::boot::vm::mem::read_buf(&mut c, mem, ptr, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let root =
                    match super::super::fs_cap::resolve(&c, crate::boot::vm::cap::FsOp::Delete, &p)
                    {
                        Ok(r) => r,
                        Err(e) => return e,
                    };
                match c.data().fs.delete(root, &p) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

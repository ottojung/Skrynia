pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "open_directory",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, ptr: u32, len: u32| -> i32 {
                let mem = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let p = match crate::boot::vm::mem::read_buf(&mut c, mem, ptr, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let root = match super::super::fs_cap::resolve(
                    &c,
                    crate::boot::vm::cap::FsOp::OpenDir,
                    &p,
                ) {
                    Ok(r) => r,
                    Err(e) => return e,
                };
                let fs = c.data().fs.clone();
                let owner = c.data().fd_scope;
                match fs.open_directory(root, &mut c.data_mut().fds, &p, owner) {
                    Ok(fd) => fd.0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "rename",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             ap: u32,
             al: u32,
             bp: u32,
             bl: u32|
             -> i32 {
                let mem = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let a = match crate::boot::vm::mem::read_buf(&mut c, mem, ap, al) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let b = match crate::boot::vm::mem::read_buf(&mut c, mem, bp, bl) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let ra =
                    match super::super::fs_cap::resolve(&c, crate::boot::vm::cap::FsOp::Rename, &a)
                    {
                        Ok(r) => r,
                        Err(e) => return e,
                    };
                let rb =
                    match super::super::fs_cap::resolve(&c, crate::boot::vm::cap::FsOp::Rename, &b)
                    {
                        Ok(r) => r,
                        Err(e) => return e,
                    };
                let root = match (ra, rb) {
                    (Some(x), Some(y)) if x == y => Some(x),
                    (Some(_), Some(_)) => return crate::boot::vm::codes::ERR_PERM,
                    (Some(x), None) | (None, Some(x)) => Some(x),
                    (None, None) => None,
                };
                match c.data().fs.rename(root, &a, &b) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

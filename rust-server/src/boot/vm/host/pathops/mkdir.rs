pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "mkdir",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             ptr: u32,
             len: u32,
             m: u32|
             -> i32 {
                let mem = match crate::boot::vm::mem::get(&mut c) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let p = match crate::boot::vm::mem::read_buf(&mut c, mem, ptr, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let mode = crate::fs::node::MkdirMode {
                    create: m & 4 != 0,
                    exclusive: m & 8 != 0,
                };
                match c.data().fs.mkdir(&p, mode) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

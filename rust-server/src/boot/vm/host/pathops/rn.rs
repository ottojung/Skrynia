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
                match c.data().fs.rename(&a, &b) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::err::map(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

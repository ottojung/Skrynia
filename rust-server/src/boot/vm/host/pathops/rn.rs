type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "rename",
            |mut c: wasmtime::Caller<'_, Vm>, ap: u64, al: u64, bp: u64, bl: u64| -> i32 {
                let a = match crate::boot::vm::host::ram::args::read(&mut c, ap, al) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let b = match crate::boot::vm::host::ram::args::read(&mut c, bp, bl) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let ra = match super::super::fs_root::resolve(&a) {
                    Ok(r) => r,
                    Err(e) => return e,
                };
                let rb = match super::super::fs_root::resolve(&b) {
                    Ok(r) => r,
                    Err(e) => return e,
                };
                let root = match (ra, rb) {
                    (Some(x), Some(y)) if x == y => Some(x),
                    (Some(_), Some(_)) => return crate::boot::vm::host::ram::args::E_NOSUP,
                    (Some(x), None) | (None, Some(x)) => Some(x),
                    (None, None) => None,
                };
                match c.data().fs.rename(root, &a, &b) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::host::ram::args::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

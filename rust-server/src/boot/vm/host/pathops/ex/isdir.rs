type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "is_dir",
            |mut c: wasmtime::Caller<'_, Vm>, ptr: u64, len: u64| -> i32 {
                let p = match crate::boot::vm::host::ram::args::read(&mut c, ptr, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let root = match super::super::super::fs_root::resolve(&p) {
                    Ok(r) => r,
                    Err(e) => return e,
                };
                match c.data().fs.is_dir(root, &p) {
                    Ok(v) => v as i32,
                    Err(e) => crate::boot::vm::host::ram::args::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

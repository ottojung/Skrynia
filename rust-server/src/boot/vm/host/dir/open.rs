type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "open_directory",
            |mut c: wasmtime::Caller<'_, Vm>, ptr: u64, len: u64| -> i32 {
                let p = match crate::boot::vm::host::memory::read(&mut c, ptr, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let root = match super::super::fs_root::resolve(&p) {
                    Ok(r) => r,
                    Err(e) => return e,
                };
                let fs = c.data().fs.clone();
                match fs.open_directory(root, &mut c.data_mut().fds, &p) {
                    Ok(fd) => fd.0,
                    Err(e) => crate::boot::vm::host::memory::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

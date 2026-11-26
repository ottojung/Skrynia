type Vm = crate::boot::vm::state::Vm;

pub fn add(linker: &mut wasmtime::Linker<Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "mkdir",
            |mut c: wasmtime::Caller<'_, Vm>, ptr: u64, len: u64, m: u32| -> i32 {
                let p = match crate::boot::vm::host::ram::args::read(&mut c, ptr, len) {
                    Ok(x) => x,
                    Err(e) => return e,
                };
                let root = match super::super::fs_cap::resolve(
                    &c,
                    crate::boot::vm::cap::FsOp::Mkdir,
                    &p,
                ) {
                    Ok(r) => r,
                    Err(e) => return e,
                };
                let mode = crate::fs::node::MkdirMode {
                    create: m & 4 != 0,
                    exclusive: m & 8 != 0,
                };
                match c.data().fs.mkdir(root, &p, mode) {
                    Ok(()) => 0,
                    Err(e) => crate::boot::vm::host::ram::args::map_fs(e),
                }
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

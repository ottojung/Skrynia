mod cas;

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "open_ram",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>| -> i32 {
                let fs = c.data().fs.clone();
                fs.open_ram(&mut c.data_mut().fds).0
            },
        )
        .map_err(|e| e.to_string())?;
    cas::add(linker)
}

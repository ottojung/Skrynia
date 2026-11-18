mod cas;

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "open_ram",
            |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>| -> i32 {
                c.data().fs.open_ram().0
            },
        )
        .map_err(|e| e.to_string())?;
    cas::add(linker)
}

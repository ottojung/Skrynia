mod run;
use run::run;
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "run_program1",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             in_fd: i32,
             out_fd: i32|
             -> i32 { run(&mut c, in_fd, out_fd) },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

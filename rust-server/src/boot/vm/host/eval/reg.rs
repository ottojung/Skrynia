use super::run::eval;
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "eval",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             cp: u32,
             cl: u32,
             in_fd: i32,
             out_fd: i32,
             cb: i32|
             -> i32 { eval(&mut c, cp, cl, in_fd, out_fd, cb, false) },
        )
        .map_err(|e| e.to_string())?;
    linker
        .func_wrap(
            "skrynia",
            "eval_delegate",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             cp: u32,
             cl: u32,
             in_fd: i32,
             out_fd: i32,
             cb: i32|
             -> i32 { eval(&mut c, cp, cl, in_fd, out_fd, cb, true) },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

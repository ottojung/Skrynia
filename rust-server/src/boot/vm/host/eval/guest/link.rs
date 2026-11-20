pub fn add(
    linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>,
    cb: Option<wasmtime::Func>,
) -> Result<(), i32> {
    crate::boot::vm::host::fd::add_stream(linker).map_err(|_| crate::boot::vm::codes::ERR_FAULT)?;
    match cb {
        None => {
            linker
                .func_wrap(
                    "skrynia",
                    "invoke",
                    |_c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
                     _: u32,
                     _: u32,
                     _: u32,
                     _: u32|
                     -> i32 { crate::boot::vm::codes::ERR_PERM },
                )
                .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
        }
        Some(cb_func) => {
            linker
                .func_wrap(
                    "skrynia",
                    "invoke",
                    move |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
                          rp: u32,
                          rl: u32,
                          dp: u32,
                          dc: u32|
                          -> i32 {
                        super::inv::call(&mut c, cb_func, rp, rl, dp, dc)
                    },
                )
                .map_err(|_| crate::boot::vm::codes::ERR_INVALID)?;
        }
    }
    Ok(())
}

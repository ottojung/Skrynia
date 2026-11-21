mod exec;
mod fin;
mod go;
mod prep;
mod run;
pub(super) fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "eval_buffered",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             cp: u32,
             cl: u32,
             rp: u32,
             rl: u32,
             dp: u32,
             dc: u32,
             cb: i32|
             -> i32 {
                exec::eval(
                    &mut c,
                    prep::Args {
                        cp,
                        cl,
                        req_ptr: rp,
                        req_len: rl,
                        res_ptr: dp,
                        res_cap: dc,
                        cb_idx: cb,
                    },
                )
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

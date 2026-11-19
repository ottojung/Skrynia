mod cb;
mod check;
mod child;
mod comp;
mod run;
pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia",
            "eval",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             cp: u32,
             cl: u32,
             ip: u32,
             il: u32,
             cb: i32,
             op: u32,
             oc: u32|
             -> i32 {
                run::eval(
                    &mut c,
                    run::Args {
                        code_ptr: cp,
                        code_len: cl,
                        in_ptr: ip,
                        in_len: il,
                        cb_idx: cb,
                        out_ptr: op,
                        out_cap: oc,
                    },
                )
            },
        )
        .map_err(|e| e.to_string())?;
    Ok(())
}

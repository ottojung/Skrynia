mod binding;
mod comp;
mod entry;
mod func;
mod link;
mod run;
mod snapshot;
mod start;

pub fn add(linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>) -> Result<(), String> {
    linker
        .func_wrap(
            "skrynia_v3_1",
            "eval",
            |mut c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>,
             code: u64,
             code_size: u64,
             function: i32,
             arg0: i32,
             arg1: i32|
             -> i64 { run::eval(&mut c, code, code_size, function, arg0, arg1) },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
}

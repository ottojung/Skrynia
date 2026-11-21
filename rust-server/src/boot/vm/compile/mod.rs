mod child;
mod fds;
mod io;
mod probe;
mod prog;
mod run;
mod slots;
mod worker;

pub fn bounded(engine: &wasmtime::Engine, code: &[u8]) -> Result<wasmtime::Module, &'static str> {
    run::bounded(engine, code)
}

pub fn trusted(engine: &wasmtime::Engine, code: &[u8]) -> Result<wasmtime::Module, &'static str> {
    if code.len() > crate::boot::vm::state::MAX_MODULE_BYTES {
        return Err("limit");
    }
    wasmtime::Module::new(engine, code).map_err(|_| "invalid")
}

pub fn engine() -> Result<wasmtime::Engine, String> {
    wasmtime::Engine::new(&crate::boot::limits::engine_config()?).map_err(|e| format!("{e}"))
}

pub fn worker_main() -> ! {
    worker::main()
}

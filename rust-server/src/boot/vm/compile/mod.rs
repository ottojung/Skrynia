mod child;
mod prog;
mod run;
mod worker;

pub fn bounded(engine: &wasmtime::Engine, code: &[u8]) -> Result<wasmtime::Module, &'static str> {
    run::bounded(engine, code)
}

pub fn engine() -> Result<wasmtime::Engine, String> {
    wasmtime::Engine::new(&crate::boot::limits::engine_config()?).map_err(|e| format!("{e}"))
}

pub fn worker_main() -> ! {
    worker::main()
}

use wasmtime::Config;

pub const MAX_WASM_STACK: usize = 512 * 1024;
pub const MAX_IO: u32 = 64 * 1024 * 1024;
pub const IO_CHUNK: u32 = 64 * 1024;
pub const MAX_COMPILE_MS: u64 = 3500;

pub fn engine_config() -> Result<Config, String> {
    let mut cfg = Config::new();
    cfg.consume_fuel(true);
    cfg.max_wasm_stack(MAX_WASM_STACK);
    Ok(cfg)
}

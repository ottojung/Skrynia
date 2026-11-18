use wasmtime::Config;

pub const DEFAULT_FUEL: u64 = 10_000_000;
pub const MAX_WASM_STACK: usize = 512 * 1024;

pub fn engine_config() -> Result<Config, String> {
    let mut cfg = Config::new();
    cfg.consume_fuel(true);
    cfg.max_wasm_stack(MAX_WASM_STACK);
    Ok(cfg)
}

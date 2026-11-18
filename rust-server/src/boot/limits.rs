use wasmtime::Config;

pub fn engine_config() -> Result<Config, String> {
    let mut cfg = Config::new();
    cfg.consume_fuel(true);
    Ok(cfg)
}

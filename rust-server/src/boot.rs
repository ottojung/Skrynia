use std::path::Path;
use std::sync::Arc;
use wasmtime::{Config, Engine, Module};

pub struct Kernel {
    engine: Engine,
    root: Module,
    ram: Arc<super::ram::Ram>,
}

impl Kernel {
    pub fn start(root_wasm: &Path) -> Result<Self, String> {
        let mut cfg = Config::new();
        cfg.consume_fuel(true);
        let engine = Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let bytes = std::fs::read(root_wasm).map_err(|e| format!("read root: {e}"))?;
        let root = Module::new(&engine, &bytes).map_err(|e| format!("compile root: {e}"))?;
        Ok(Self {
            engine,
            root,
            ram: Arc::new(super::ram::Ram::new()),
        })
    }
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
    pub fn root(&self) -> &Module {
        &self.root
    }
    pub fn ram(&self) -> &Arc<super::ram::Ram> {
        &self.ram
    }
}

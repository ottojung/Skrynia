mod exec;
mod kernel_api;
mod limits;
pub mod vm;
pub use exec::Request;
use std::path::Path;
use std::sync::Arc;
use wasmtime::{Engine, Module};

pub struct Kernel {
    engine: Engine,
    root: Module,
    ram: Arc<crate::ram::Ram>,
    fs: Arc<crate::fs::Fs>,
    cache: Arc<vm::state::Cache>,
}

impl Kernel {
    pub fn start(root_wasm: &Path, data_dir: &Path) -> Result<Self, String> {
        let cfg = limits::engine_config()?;
        let engine = Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let bytes = std::fs::read(root_wasm).map_err(|e| format!("read root: {e}"))?;
        let root = Module::new(&engine, &bytes).map_err(|e| format!("compile root: {e}"))?;
        let data = Arc::new(crate::fs::root::Root::open(data_dir)?);
        let ram = Arc::new(crate::ram::Ram::new());
        let fs = Arc::new(crate::fs::Fs::new(data, ram.clone()));
        let cache = Arc::new(vm::state::Cache::new());
        Ok(Self {
            engine,
            root,
            ram,
            fs,
            cache,
        })
    }
}

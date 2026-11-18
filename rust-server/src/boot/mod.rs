mod exec;
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
        Ok(Self {
            engine,
            root,
            ram,
            fs,
        })
    }
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
    pub fn root(&self) -> &Module {
        &self.root
    }
    pub fn ram(&self) -> &Arc<crate::ram::Ram> {
        &self.ram
    }
    pub fn fs(&self) -> &Arc<crate::fs::Fs> {
        &self.fs
    }
    pub fn prepare(&self, fuel: u64) -> Result<Request, String> {
        exec::prepare(&self.engine, &self.root, fuel)
    }
    pub fn invoke(&self, body: &[u8]) -> Result<Vec<u8>, i32> {
        vm::run::invoke(self, body)
    }
}

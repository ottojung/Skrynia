use super::validate;
use std::path::Path;
use std::sync::Arc;
use wasmtime::{Engine, Module};

pub struct Kernel {
    pub(super) engine: Engine,
    pub(super) root: Module,
    pub(super) program1: Option<Module>,
    pub(super) ram: Arc<crate::ram::Ram>,
    pub(super) fs: Arc<crate::fs::Fs>,
    pub(super) cache: Arc<super::vm::state::Cache>,
}

impl Kernel {
    pub fn start(root_wasm: &Path, data_dir: &Path) -> Result<Self, String> {
        Self::start_with(root_wasm, data_dir, None)
    }
    pub fn start_with(
        root_wasm: &Path,
        data_dir: &Path,
        program1: Option<&Path>,
    ) -> Result<Self, String> {
        let cfg = super::limits::engine_config()?;
        let engine = Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let bytes = std::fs::read(root_wasm).map_err(|e| format!("read root: {e}"))?;
        let root = Module::new(&engine, &bytes).map_err(|e| format!("compile root: {e}"))?;
        validate::check(&root)?;
        let program1 = match program1 {
            Some(p) => {
                let b = std::fs::read(p).map_err(|e| format!("read program1: {e}"))?;
                let m = Module::new(&engine, &b).map_err(|e| format!("compile program1: {e}"))?;
                validate::check_program1(&m)?;
                Some(m)
            }
            None => None,
        };
        let data = Arc::new(crate::fs::root::Root::open(data_dir)?);
        let ram = Arc::new(crate::ram::Ram::new());
        let fs = Arc::new(crate::fs::Fs::new(data, ram.clone()));
        let cache = Arc::new(super::vm::state::Cache::new());
        Ok(Self {
            engine,
            root,
            program1,
            ram,
            fs,
            cache,
        })
    }
}

use super::{Arc, Engine, Kernel, Module, Path, validate};

impl Kernel {
    pub fn start_with_scope(
        root_wasm: &Path,
        data_dir: &Path,
        program1: Option<&Path>,
        storage: Option<&Path>,
    ) -> Result<Self, String> {
        if storage.is_some() && program1.is_none() {
            return Err("Program 1 storage requires Program 1 WASM".into());
        }
        let cfg = crate::boot::limits::engine_config()?;
        let engine = Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let bytes = std::fs::read(root_wasm).map_err(|e| format!("read root: {e}"))?;
        let root = Module::new(&engine, &bytes).map_err(|e| format!("compile root: {e}"))?;
        validate::check(&root)?;
        let program1 = match program1 {
            Some(path) => {
                let bytes = std::fs::read(path).map_err(|e| format!("read program1: {e}"))?;
                let module =
                    Module::new(&engine, &bytes).map_err(|e| format!("compile program1: {e}"))?;
                validate::check_program1(&module, storage.is_some())?;
                Some(module)
            }
            None => None,
        };
        let data = Arc::new(crate::fs::root::Root::open(data_dir)?);
        let ram = Arc::new(crate::ram::Ram::new());
        let fs = Arc::new(crate::fs::Fs::new(data, ram.clone()));
        let program1_storage = match storage {
            Some(path) => {
                let root = Arc::new(crate::fs::root::Root::open(path)?);
                Some(Arc::new(crate::fs::Fs::new(root, ram.clone())))
            }
            None => None,
        };
        let cache = Arc::new(crate::boot::vm::state::Cache::new());
        Ok(Self {
            engine,
            root,
            program1,
            program1_storage,
            ram,
            fs,
            cache,
        })
    }
}

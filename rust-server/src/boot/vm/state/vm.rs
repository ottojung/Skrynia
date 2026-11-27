use std::sync::Arc;

impl super::Vm {
    pub fn new(
        engine: wasmtime::Engine,
        fs: Arc<crate::fs::Fs>,
        ram: Arc<crate::ram::Ram>,
        cache: Arc<super::Cache>,
    ) -> Self {
        Self {
            engine,
            fs,
            ram,
            fds: crate::fs::table::Table::new(),
            cache,
            depth: 0,
        }
    }
}

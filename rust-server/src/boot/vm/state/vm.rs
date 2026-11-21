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
            frames: Vec::new(),
            cache,
            depth: 0,
            guest: None,
            mutation_held: false,
            limits: wasmtime::StoreLimitsBuilder::new()
                .memory_size(64 * 1024 * 1024)
                .table_elements(10_000)
                .instances(16)
                .tables(16)
                .memories(16)
                .build(),
            providers: Vec::new(),
            cur: None,
        }
    }
}

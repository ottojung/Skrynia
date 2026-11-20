mod cache;
mod frame;
mod mutation;
mod root;
pub use cache::{Cache, MAX_MODULE_BYTES};
pub use frame::Frame;
use std::sync::Arc;

pub struct Vm {
    pub engine: wasmtime::Engine,
    pub fs: Arc<crate::fs::Fs>,
    pub ram: Arc<crate::ram::Ram>,
    pub fds: crate::fs::table::Table<crate::fs::store::Item>,
    pub frames: Vec<Frame>,
    pub cache: Arc<Cache>,
    pub depth: u32,
    pub mutation_held: bool,
    pub program1: Option<wasmtime::Module>,
    pub program1_storage: Option<Arc<crate::fs::Fs>>,
    pub limits: wasmtime::StoreLimits,
}

impl Vm {
    pub fn new(
        engine: wasmtime::Engine,
        fs: Arc<crate::fs::Fs>,
        ram: Arc<crate::ram::Ram>,
        cache: Arc<Cache>,
        program1: Option<wasmtime::Module>,
    ) -> Self {
        Self {
            engine,
            fs,
            ram,
            fds: crate::fs::table::Table::new(),
            frames: Vec::new(),
            cache,
            depth: 0,
            mutation_held: false,
            program1,
            program1_storage: None,
            limits: wasmtime::StoreLimitsBuilder::new()
                .memory_size(64 * 1024 * 1024)
                .table_elements(10_000)
                .instances(16)
                .tables(16)
                .memories(16)
                .build(),
        }
    }
}

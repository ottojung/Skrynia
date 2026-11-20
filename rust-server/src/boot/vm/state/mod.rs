mod cache;
mod cap;
mod frame;
mod mutation;
pub use cache::{Cache, MAX_MODULE_BYTES};
pub use cap::GuestCap;
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
    pub guest: Option<GuestCap>,
    pub mutation_held: bool,
    pub limits: wasmtime::StoreLimits,
}

impl Vm {
    pub fn new(
        engine: wasmtime::Engine,
        fs: Arc<crate::fs::Fs>,
        ram: Arc<crate::ram::Ram>,
        cache: Arc<Cache>,
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
        }
    }
    pub fn root_mut(&mut self) -> &mut Frame {
        &mut self.frames[0]
    }
}

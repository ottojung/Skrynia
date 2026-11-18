mod cache;
mod frame;
pub use cache::Cache;
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
        }
    }
    pub fn root_mut(&mut self) -> &mut Frame {
        &mut self.frames[0]
    }
}

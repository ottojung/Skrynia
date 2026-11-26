mod cache;
mod vm;
pub use cache::{Cache, MAX_MODULE_BYTES};
use std::sync::Arc;

pub struct Vm {
    pub engine: wasmtime::Engine,
    pub fs: Arc<crate::fs::Fs>,
    pub ram: Arc<crate::ram::Ram>,
    pub fds: crate::fs::table::Table<crate::fs::store::Item>,
    pub cache: Arc<Cache>,
    pub depth: u32,
}

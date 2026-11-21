mod cache;
mod cap;
mod ctx;
mod frame;
mod mutation;
mod vm;
pub use cache::{Cache, MAX_MODULE_BYTES};
pub use cap::GuestCap;
pub use ctx::ProviderCtx;
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
    pub providers: Vec<ProviderCtx>,
    pub cur: Option<wasmtime::Instance>,
    pub fs_auth: crate::boot::vm::cap::FsAuth,
    pub fs_delegate: crate::boot::vm::cap::FsAuth,
}

impl Vm {
    pub fn root_mut(&mut self) -> &mut Frame {
        &mut self.frames[0]
    }
}

mod cache;
mod cap;
mod ctx;
mod frame;
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
    pub providers: Vec<ProviderCtx>,
    pub cur: Option<wasmtime::Instance>,
    pub fs_auth: crate::boot::vm::cap::FsAuth,
    pub fs_delegate: crate::boot::vm::cap::FsAuth,
    pub fd_scope: u32,
    pub fd_next_scope: u32,
}

impl Vm {
    pub fn pop_provider(&mut self) {
        if let Some(prov) = self.providers.pop() {
            self.fds.remove_owner(prov.callback_scope);
        }
    }
    pub fn root_mut(&mut self) -> &mut Frame {
        &mut self.frames[0]
    }
}

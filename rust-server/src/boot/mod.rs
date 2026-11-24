mod ancestry;
mod distinct;
mod exec;
mod kernel_api;
mod limits;
mod pipe;
mod root;
mod validate;
pub mod vm;
pub use exec::Request;
use std::path::Path;
use std::sync::Arc;
use wasmtime::{Engine, Module};

pub struct Kernel {
    engine: Engine,
    root: Module,
    ram: Arc<crate::ram::Ram>,
    fs: Arc<crate::fs::Fs>,
    cache: Arc<vm::state::Cache>,
    pre: wasmtime::InstancePre<vm::state::Vm>,
    version: serde_json::Value,
}

impl Kernel {
    pub fn start(root_wasm: &Path, operator_dir: &Path, user_dir: &Path) -> Result<Self, String> {
        Self::start_with_syscalls(root_wasm, operator_dir, user_dir, crate::Syscalls::empty())
    }
    pub fn start_with_roots(
        root_wasm: &Path,
        operator_dir: &Path,
        user_dir: &Path,
    ) -> Result<Self, String> {
        Self::start(root_wasm, operator_dir, user_dir)
    }
}

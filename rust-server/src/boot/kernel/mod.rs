mod init;
use super::validate;
use std::path::Path;
use std::sync::Arc;
use wasmtime::{Engine, Module};

pub struct Kernel {
    pub(super) engine: Engine,
    pub(super) root: Module,
    pub(super) program1: Option<Module>,
    pub(super) program1_storage: Option<Arc<crate::fs::Fs>>,
    pub(super) ram: Arc<crate::ram::Ram>,
    pub(super) fs: Arc<crate::fs::Fs>,
    pub(super) cache: Arc<super::vm::state::Cache>,
}
impl Kernel {
    pub fn start(root: &Path, data: &Path) -> Result<Self, String> {
        Self::start_with(root, data, None)
    }
    pub fn start_with(root: &Path, data: &Path, p1: Option<&Path>) -> Result<Self, String> {
        Self::start_with_scope(root, data, p1, None)
    }
}

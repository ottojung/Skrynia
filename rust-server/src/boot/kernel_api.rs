use super::{Kernel, Request, exec, vm};
use std::sync::Arc;
use wasmtime::{Engine, Module};

impl Kernel {
    pub fn engine(&self) -> &Engine {
        &self.engine
    }
    pub fn root(&self) -> &Module {
        &self.root
    }
    pub fn ram(&self) -> &Arc<crate::ram::Ram> {
        &self.ram
    }
    pub fn fs(&self) -> &Arc<crate::fs::Fs> {
        &self.fs
    }
    pub fn cached_modules(&self) -> usize {
        self.cache.len()
    }
    pub fn cache(&self) -> &Arc<vm::state::Cache> {
        &self.cache
    }
    pub fn prepare(&self, fuel: u64) -> Result<Request, String> {
        exec::prepare(&self.engine, &self.root, fuel)
    }
    pub fn invoke(&self, body: &[u8]) -> Result<Vec<u8>, i32> {
        vm::run::invoke(self, body)
    }
}

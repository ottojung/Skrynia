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
    pub fn program1(&self) -> Option<&Module> {
        self.program1.as_ref()
    }
    pub fn program1_storage(&self) -> Option<&Arc<crate::fs::Fs>> {
        self.program1_storage.as_ref()
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
        let sink = super::pipe::VecSink::default();
        self.invoke_stream(
            Box::new(std::io::Cursor::new(body.to_vec())),
            16 * 1024 * 1024,
            Box::new(sink.clone()),
        )?;
        Ok(sink.result())
    }
    pub fn invoke_stream(
        &self,
        input: Box<dyn std::io::Read + Send>,
        max_bytes: u64,
        output: Box<dyn std::io::Write + Send>,
    ) -> Result<(), i32> {
        vm::run::invoke(self, input, max_bytes, output)
    }
}

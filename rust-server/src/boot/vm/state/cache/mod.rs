mod exec;
mod flight;
mod guard;
use flight::Flight;
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
const MAX_ENTRIES: usize = 64;
pub const MAX_MODULE_BYTES: usize = 8 * 1024 * 1024;
pub struct Cache {
    ready: Mutex<HashMap<[u8; 32], wasmtime::Module>>,
    inflight: Mutex<HashMap<[u8; 32], Arc<Flight>>>,
}
impl Cache {
    pub fn new() -> Self {
        Self {
            ready: Mutex::new(HashMap::new()),
            inflight: Mutex::new(HashMap::new()),
        }
    }
    fn key(code: &[u8]) -> [u8; 32] {
        Sha256::digest(code).into()
    }
    pub fn get(&self, code: &[u8]) -> Option<wasmtime::Module> {
        self.ready.lock().unwrap().get(&Self::key(code)).cloned()
    }
    pub fn put(&self, code: &[u8], module: wasmtime::Module) {
        let mut m = self.ready.lock().unwrap();
        if m.len() >= MAX_ENTRIES {
            m.clear();
        }
        m.insert(Self::key(code), module);
    }
    pub fn len(&self) -> usize {
        self.ready.lock().unwrap().len()
    }
    pub fn is_empty(&self) -> bool {
        self.ready.lock().unwrap().is_empty()
    }
}
impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

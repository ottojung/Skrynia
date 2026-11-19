use sha2::{Digest, Sha256};
use std::{collections::HashMap, sync::Mutex};

const MAX_ENTRIES: usize = 64;
pub const MAX_MODULE_BYTES: usize = 8 * 1024 * 1024;

pub struct Cache {
    map: Mutex<HashMap<[u8; 32], wasmtime::Module>>,
}
impl Cache {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
        }
    }
    fn key(code: &[u8]) -> [u8; 32] {
        Sha256::digest(code).into()
    }
    pub fn get(&self, code: &[u8]) -> Option<wasmtime::Module> {
        self.map.lock().unwrap().get(&Self::key(code)).cloned()
    }
    pub fn put(&self, code: &[u8], module: wasmtime::Module) {
        let mut map = self.map.lock().unwrap();
        if map.len() >= MAX_ENTRIES {
            map.clear();
        }
        map.insert(Self::key(code), module);
    }
    pub fn len(&self) -> usize {
        self.map.lock().unwrap().len()
    }
}

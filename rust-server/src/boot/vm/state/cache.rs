use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::sync::Mutex;
use wasmtime::Module;

const MAX_ENTRIES: usize = 32;
pub struct Cache {
    modules: Mutex<VecDeque<([u8; 32], Module)>>,
}
impl Cache {
    pub fn new() -> Self {
        Self {
            modules: Mutex::new(VecDeque::new()),
        }
    }
    fn key(bytes: &[u8]) -> [u8; 32] {
        Sha256::digest(bytes).into()
    }
    pub fn get(&self, code: &[u8]) -> Option<Module> {
        let id = Self::key(code);
        let mut modules = self.modules.lock().unwrap();
        let pos = modules.iter().position(|(key, _)| key == &id)?;
        let item = modules.remove(pos)?;
        let result = item.1.clone();
        modules.push_front(item);
        Some(result)
    }
    pub fn put(&self, code: &[u8], module: Module) {
        let id = Self::key(code);
        let mut modules = self.modules.lock().unwrap();
        if let Some(pos) = modules.iter().position(|(key, _)| key == &id) {
            modules.remove(pos);
        }
        modules.push_front((id, module));
        modules.truncate(MAX_ENTRIES);
    }
    pub fn len(&self) -> usize {
        self.modules.lock().unwrap().len()
    }
}

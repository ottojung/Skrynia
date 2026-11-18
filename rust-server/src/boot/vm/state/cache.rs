pub struct Cache {
    map: std::sync::Mutex<std::collections::HashMap<Vec<u8>, wasmtime::Module>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            map: std::sync::Mutex::new(std::collections::HashMap::new()),
        }
    }
    pub fn get(&self, code: &[u8]) -> Option<wasmtime::Module> {
        self.map.lock().unwrap().get(code).cloned()
    }
    pub fn put(&self, code: Vec<u8>, module: wasmtime::Module) {
        self.map.lock().unwrap().insert(code, module);
    }
}

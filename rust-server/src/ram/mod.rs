use std::sync::Mutex;
mod cas;
mod io;

#[derive(Debug, Default)]
pub struct Ram {
    bytes: Mutex<Vec<u8>>,
}

impl Ram {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn len(&self) -> usize {
        self.bytes.lock().unwrap().len()
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn truncate(&self, size: usize) {
        let mut b = self.bytes.lock().unwrap();
        b.resize(size, 0);
    }
}

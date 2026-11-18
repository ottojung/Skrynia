use std::sync::Mutex;
mod cas;
mod io;

pub const MAX_RAM_BYTES: usize = 64 * 1024 * 1024;

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
    pub fn truncate(&self, size: usize) -> Result<(), &'static str> {
        if size > MAX_RAM_BYTES {
            return Err("nospace");
        }
        let mut b = self.bytes.lock().unwrap();
        let extra = size.saturating_sub(b.len());
        b.try_reserve(extra).map_err(|_| "nospace")?;
        b.resize(size, 0);
        Ok(())
    }
}

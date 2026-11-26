use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct BufferedOutput {
    inner: Arc<Mutex<Inner>>,
}
struct Inner {
    data: Vec<u8>,
    cap: usize,
}
impl BufferedOutput {
    pub fn new(cap: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                data: Vec::new(),
                cap,
            })),
        }
    }
    pub fn write(&mut self, src: &[u8]) -> Result<usize, &'static str> {
        let mut g = self.inner.lock().map_err(|_| "io")?;
        if g.data.len().saturating_add(src.len()) > g.cap {
            return Err("limit");
        }
        g.data.extend_from_slice(src);
        Ok(src.len())
    }
    pub fn write1(&mut self, byte: u8) -> Result<(), &'static str> {
        self.write(&[byte]).map(|_| ())
    }
    pub fn take(&self) -> Vec<u8> {
        match self.inner.lock() {
            Ok(mut g) => std::mem::take(&mut g.data),
            Err(_) => Vec::new(),
        }
    }
}

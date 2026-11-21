use std::{
    io,
    sync::{Arc, Mutex},
};
#[derive(Clone, Default)]
pub struct VecSink(pub Arc<Mutex<Vec<u8>>>);
impl io::Write for VecSink {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0
            .lock()
            .map_err(|_| io::Error::other("poisoned"))?
            .extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl VecSink {
    pub fn result(&self) -> Vec<u8> {
        self.0.lock().unwrap().clone()
    }
}

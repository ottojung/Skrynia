use crate::{Fail, Result};
use axum::http::StatusCode;
use std::sync::Arc;
use tokio::sync::Semaphore;
pub struct Upload {
    pub bytes: Vec<u8>,
    sem: Arc<Semaphore>,
    reserved: usize,
}
impl Upload {
    pub fn empty(sem: Arc<Semaphore>) -> Self {
        Self {
            bytes: Vec::new(),
            sem,
            reserved: 0,
        }
    }
    pub fn append(&mut self, chunk: &[u8]) -> Result<()> {
        if chunk.is_empty() {
            return Ok(());
        }
        let amount = chunk.len().try_into().map_err(|_| memory_full())?;
        let permit = self
            .sem
            .clone()
            .try_acquire_many_owned(amount)
            .map_err(|_| memory_full())?;
        permit.forget();
        self.reserved += chunk.len();
        self.bytes
            .try_reserve_exact(chunk.len())
            .map_err(|_| memory_full())?;
        self.bytes.extend_from_slice(chunk);
        Ok(())
    }
}
impl Drop for Upload {
    fn drop(&mut self) {
        if self.reserved > 0 {
            self.sem.add_permits(self.reserved);
        }
    }
}
fn memory_full() -> Fail {
    Fail(StatusCode::SERVICE_UNAVAILABLE, "upload_memory_full")
}

use axum::body::Bytes;
use std::io::{self, Read};
use tokio::sync::mpsc;

pub struct Input {
    receiver: mpsc::Receiver<io::Result<Bytes>>,
    pending: Bytes,
    error: Option<io::ErrorKind>,
}
impl Input {
    pub fn new(receiver: mpsc::Receiver<io::Result<Bytes>>) -> Self {
        Self {
            receiver,
            pending: Bytes::new(),
            error: None,
        }
    }
}
impl Read for Input {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        if dst.is_empty() {
            return Ok(0);
        }
        if let Some(kind) = self.error {
            return Err(io::Error::from(kind));
        }
        while self.pending.is_empty() {
            match self.receiver.blocking_recv() {
                Some(Ok(chunk)) => self.pending = chunk,
                Some(Err(e)) => {
                    self.error = Some(e.kind());
                    return Err(e);
                }
                None => return Ok(0),
            }
        }
        let n = dst.len().min(self.pending.len());
        dst[..n].copy_from_slice(&self.pending.split_to(n));
        Ok(n)
    }
}

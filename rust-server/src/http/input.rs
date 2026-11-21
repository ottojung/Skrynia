use super::term;
use axum::body::Bytes;
use std::io::{self, Read};
use tokio::sync::mpsc;

pub struct Input {
    receiver: mpsc::Receiver<io::Result<Bytes>>,
    pending: Bytes,
    term: term::Term,
}
impl Input {
    pub fn new(receiver: mpsc::Receiver<io::Result<Bytes>>, term: term::Term) -> Self {
        Self {
            receiver,
            pending: Bytes::new(),
            term,
        }
    }
}
impl Read for Input {
    fn read(&mut self, dst: &mut [u8]) -> io::Result<usize> {
        if dst.is_empty() {
            return Ok(0);
        }
        if let Some(e) = term::get(&self.term) {
            return Err(e);
        }
        while self.pending.is_empty() {
            match self.receiver.blocking_recv() {
                Some(Ok(chunk)) => self.pending = chunk,
                Some(Err(e)) => {
                    term::set(&self.term, &e);
                    return Err(e);
                }
                None => return term::get(&self.term).map_or(Ok(0), Err),
            }
        }
        let n = dst.len().min(self.pending.len());
        dst[..n].copy_from_slice(&self.pending.split_to(n));
        Ok(n)
    }
}

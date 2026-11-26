use axum::body::Bytes;
use std::io::{self, Write};
use tokio::sync::mpsc;
use tokio::time::{Instant, timeout_at};

pub type Chunk = io::Result<Bytes>;
pub struct Output(pub mpsc::Sender<Chunk>, pub Instant);

impl Write for Output {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        if src.is_empty() {
            return Ok(0);
        }
        let count = src.len().min(64 * 1024);
        if Instant::now() >= self.1 {
            return Err(deadline());
        }
        let chunk = Ok(Bytes::copy_from_slice(&src[..count]));
        match self.0.try_send(chunk) {
            Ok(()) => Ok(count),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(closed()),
            Err(mpsc::error::TrySendError::Full(chunk)) => {
                let send = async { timeout_at(self.1, self.0.send(chunk)).await };
                tokio::runtime::Handle::current()
                    .block_on(send)
                    .map_err(|_| deadline())?
                    .map_err(|_| closed())?;
                Ok(count)
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
fn deadline() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "response deadline")
}
fn closed() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "response closed")
}

mod stream;
pub use stream::body;
#[cfg(test)]
mod tests;

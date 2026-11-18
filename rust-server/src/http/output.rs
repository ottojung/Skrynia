use axum::body::Bytes;
use std::io::{self, Write};
use tokio::sync::mpsc;

pub type Chunk = io::Result<Bytes>;
pub struct Output(pub mpsc::Sender<Chunk>);
impl Write for Output {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        if src.is_empty() {
            return Ok(0);
        }
        let count = src.len().min(64 * 1024);
        self.0
            .blocking_send(Ok(Bytes::copy_from_slice(&src[..count])))
            .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "response closed"))?;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

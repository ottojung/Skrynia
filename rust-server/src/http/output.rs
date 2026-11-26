use axum::body::Bytes;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::mpsc;

pub type Chunk = io::Result<Bytes>;
pub type Abort = Arc<AtomicBool>;

pub fn abort() -> Abort {
    Arc::new(AtomicBool::new(false))
}

pub struct Output(pub mpsc::Sender<Chunk>, pub Abort);

impl Write for Output {
    fn write(&mut self, src: &[u8]) -> io::Result<usize> {
        if src.is_empty() {
            return Ok(0);
        }
        let count = src.len().min(64 * 1024);
        if self.1.load(Ordering::Relaxed) {
            return Err(io::Error::other("response deadline"));
        }
        // The VM runs in spawn_blocking, so a bounded blocking send is safe here.
        // It applies the same backpressure as the old try_send + 2ms sleep loop,
        // but wakes as soon as the consumer drains a slot instead of sleeping a
        // fixed 2ms per byte (which capped byte-wise write1 output near 1KB/s).
        let chunk = Ok(Bytes::copy_from_slice(&src[..count]));
        match self.0.blocking_send(chunk) {
            Ok(()) => Ok(count),
            Err(_) => Err(io::Error::other("response closed")),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn body(first: Bytes, receiver: mpsc::Receiver<Chunk>) -> axum::body::Body {
    let stream =
        futures_util::stream::unfold((Some(Ok(first)), receiver), |(first, mut rx)| async move {
            if let Some(chunk) = first {
                return Some((chunk, (None, rx)));
            }
            rx.recv().await.map(|chunk| (chunk, (None, rx)))
        });
    axum::body::Body::from_stream(stream)
}

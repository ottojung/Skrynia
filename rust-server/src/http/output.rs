use axum::body::Bytes;
use std::io::{self, Write};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
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
        loop {
            if self.1.load(Ordering::Relaxed) {
                return Err(io::Error::other("response deadline"));
            }
            let chunk = Ok(Bytes::copy_from_slice(&src[..count]));
            match self.0.try_send(chunk) {
                Ok(()) => return Ok(count),
                Err(mpsc::error::TrySendError::Full(_)) => {
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    return Err(io::Error::other("response closed"));
                }
            }
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

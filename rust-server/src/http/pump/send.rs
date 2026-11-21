use super::stalled;
use axum::body::Bytes;
use std::io;
use tokio::sync::mpsc;

pub async fn part(
    tx: &mpsc::Sender<io::Result<Bytes>>,
    end: tokio::time::Instant,
    b: Bytes,
) -> io::Result<()> {
    tokio::select! {
        biased;
        _ = tokio::time::sleep_until(end) => Err(stalled()),
        r = tx.send(Ok(b)) => r.map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "input closed")),
    }
}
pub fn stop(t: &super::super::term::Term, e: io::Error) -> io::Error {
    super::super::term::set(t, &e);
    e
}

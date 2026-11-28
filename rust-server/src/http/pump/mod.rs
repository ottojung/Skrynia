mod read;
mod send;
use axum::body::{Body, Bytes};
use futures_util::{Stream, StreamExt, stream};
use std::io;
use std::pin::Pin;
use tokio::{sync::mpsc, time::Duration};
#[derive(Clone, Copy)]
pub struct Limits {
    pub max: usize,
    pub inactivity: Duration,
    pub total: Duration,
}
pub fn stalled() -> io::Error {
    io::Error::new(io::ErrorKind::TimedOut, "stalled request body")
}
// A timeout is an error item, never EOF: gate must not mistake it for a
// complete body and emit a truncated 200.
pub fn inactive<S>(
    s: S,
    d: Duration,
) -> Pin<Box<dyn Stream<Item = Result<Bytes, axum::Error>> + Send>>
where
    S: Stream<Item = Result<Bytes, axum::Error>> + Unpin + Send + 'static,
{
    Box::pin(stream::unfold((s, d), |(mut s, d)| async move {
        let next = tokio::select! {
            biased;
            _ = tokio::time::sleep(d) => Some(Err(axum::Error::new(
                io::Error::from(io::ErrorKind::TimedOut),
            ))),
            n = s.next() => n,
        }?;
        Some((next, (s, d)))
    }))
}
pub async fn run(
    body: Body,
    tx: mpsc::Sender<io::Result<Bytes>>,
    t: super::term::Term,
    lim: Limits,
    buffered: bool,
) -> io::Result<()> {
    read::run(body, tx, t, lim, buffered).await
}

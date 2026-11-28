mod read;
mod send;
use axum::body::{Body, Bytes};
use std::io;
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
pub async fn run(
    body: Body,
    tx: mpsc::Sender<io::Result<Bytes>>,
    t: super::term::Term,
    lim: Limits,
    buffered: bool,
) -> io::Result<()> {
    read::run(body, tx, t, lim, buffered).await
}

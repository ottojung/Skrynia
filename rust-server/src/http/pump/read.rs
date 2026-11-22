use super::{Limits, stalled};
use axum::body::{Body, Bytes};
use futures_util::StreamExt;
use std::io;
use tokio::sync::mpsc;

async fn fail(tx: &mpsc::Sender<io::Result<Bytes>>, on: bool, e: io::Error) -> io::Error {
    if on {
        let _ = tx.send(Err(io::Error::new(e.kind(), e.to_string()))).await;
    }
    e
}
pub async fn run(body: Body, tx: mpsc::Sender<io::Result<Bytes>>, lim: Limits) -> io::Result<()> {
    let mut stream = body.into_data_stream();
    let end = tokio::time::Instant::now() + lim.total;
    let mut count = 0usize;
    let mut on = true;
    loop {
        let idle = tokio::time::sleep(lim.inactivity);
        let chunk = tokio::select! {
            biased;
            _ = tokio::time::sleep_until(end) => return Err(fail(&tx, on, stalled()).await),
            _ = idle => return Err(fail(&tx, on, stalled()).await),
            c = stream.next() => c,
        };
        let Some(chunk) = chunk else { return Ok(()) };
        let b = match chunk {
            Ok(b) => b,
            Err(_) => return Err(fail(&tx, on, io::ErrorKind::BrokenPipe.into()).await),
        };
        count = count
            .checked_add(b.len())
            .ok_or(io::ErrorKind::InvalidData)?;
        if count > lim.max {
            return Err(fail(&tx, on, io::ErrorKind::InvalidData.into()).await);
        }
        for part in b.chunks(64 * 1024) {
            if on && tx.send(Ok(Bytes::copy_from_slice(part))).await.is_err() {
                on = false;
            }
        }
    }
}

use super::{Limits, send, stalled};
use axum::body::{Body, Bytes};
use futures_util::StreamExt;
use std::io;
use tokio::sync::mpsc;

pub async fn run(
    body: Body,
    tx: mpsc::Sender<io::Result<Bytes>>,
    t: super::super::term::Term,
    lim: Limits,
    buffered: bool,
) -> io::Result<()> {
    let mut stream = body.into_data_stream();
    let end = tokio::time::Instant::now() + lim.total;
    let mut count = 0usize;
    let mut on = true;
    loop {
        let idle = tokio::time::sleep(lim.inactivity);
        let chunk = tokio::select! {
            biased;
            _ = tokio::time::sleep_until(end) => return Err(send::stop(&t, stalled())),
            _ = idle, if !buffered => return Err(send::stop(&t, stalled())),
            c = stream.next() => c,
        };
        let Some(chunk) = chunk else { return Ok(()) };
        let b = match chunk {
            Ok(b) => b,
            Err(_) => return Err(send::stop(&t, io::Error::from(io::ErrorKind::BrokenPipe))),
        };
        count = match count.checked_add(b.len()) {
            Some(n) => n,
            None => return Err(send::stop(&t, io::Error::from(io::ErrorKind::InvalidData))),
        };
        if count > lim.max {
            return Err(send::stop(&t, io::Error::from(io::ErrorKind::InvalidData)));
        }
        for part in b.chunks(64 * 1024) {
            if !on {
                continue;
            }
            match send::part(&tx, end, Bytes::copy_from_slice(part)).await {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::BrokenPipe => on = false,
                Err(e) => return Err(send::stop(&t, e)),
            }
        }
    }
}

use axum::body::{Body, Bytes};
use futures_util::StreamExt;
use std::io;
use tokio::sync::mpsc;

pub async fn run(body: Body, tx: mpsc::Sender<io::Result<Bytes>>, max: usize) -> io::Result<()> {
    let mut stream = body.into_data_stream();
    let mut count = 0usize;
    let mut active = true;
    while let Some(chunk) = stream.next().await {
        let b = match chunk {
            Ok(b) => b,
            Err(_) => {
                if active {
                    let _ = tx.send(Err(io::ErrorKind::BrokenPipe.into())).await;
                }
                return Err(io::ErrorKind::BrokenPipe.into());
            }
        };
        count = count
            .checked_add(b.len())
            .ok_or(io::ErrorKind::InvalidData)?;
        if count > max {
            if active {
                let _ = tx.send(Err(io::ErrorKind::InvalidData.into())).await;
            }
            return Err(io::ErrorKind::InvalidData.into());
        }
        for part in b.chunks(64 * 1024) {
            if active && tx.send(Ok(Bytes::copy_from_slice(part))).await.is_err() {
                active = false;
            }
        }
    }
    Ok(())
}

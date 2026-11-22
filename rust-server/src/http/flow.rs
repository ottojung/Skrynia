use super::output::Chunk;
use std::io;
use tokio::{sync::mpsc, task::JoinHandle};

// Mirrors boot::vm::codes::ERR_LIMIT; a nested import is forbidden by the
// source linter. No limit is changed: this only classifies the fail-closed
// result so the HTTP layer can answer 503 instead of a catch-all 500.
const LIMIT: i32 = -10;

fn fail(code: i32) -> io::Error {
    match code {
        LIMIT => io::ErrorKind::WouldBlock.into(),
        _ => io::Error::other("VM guest fault"),
    }
}

pub async fn supervise(
    mut pump: JoinHandle<io::Result<()>>,
    mut worker: JoinHandle<Result<(), i32>>,
    tx: mpsc::Sender<Chunk>,
) {
    tokio::select! {
        biased;
        result = &mut pump => {
            match result {
                Ok(Ok(())) => {
                    match worker.await {
                        Ok(Err(code)) => { let _ = tx.send(Err(fail(code))).await; }
                        Err(_) => { let _ = tx.send(Err(fail(-9))).await; }
                        _ => {}
                    }
                }
                Ok(Err(e)) => {
                    let _ = tx.send(Err(e)).await;
                    let _ = worker.await;
                }
                Err(_) => {
                    let _ = tx.send(Err(io::Error::other("input pump stopped"))).await;
                    let _ = worker.await;
                }
            }
        }
        result = &mut worker => {
            pump.abort();
            match result {
                Ok(Err(code)) => { let _ = tx.send(Err(fail(code))).await; }
                Err(_) => { let _ = tx.send(Err(fail(-9))).await; }
                _ => {}
            }
        }
    }
}

use super::output::Chunk;
use std::io;
use tokio::{sync::mpsc, task::JoinHandle};

pub async fn supervise(
    mut pump: JoinHandle<io::Result<()>>,
    mut worker: JoinHandle<Result<(), i32>>,
    tx: mpsc::Sender<Chunk>,
) {
    let fail = |code: i32| io::Error::other(format!("VM error {code}"));
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

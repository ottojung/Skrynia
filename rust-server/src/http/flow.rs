use super::output::Chunk;
use std::io;
use tokio::{sync::mpsc, task::JoinHandle};

pub async fn supervise(
    mut pump: JoinHandle<io::Result<()>>,
    mut worker: JoinHandle<Result<(), i32>>,
    terminal: mpsc::OwnedPermit<Chunk>,
) {
    let fail = |code: i32| io::Error::other(format!("VM error {code}"));
    tokio::select! {
        biased;
        result = &mut pump => {
            match result {
                Ok(Ok(())) => {
                    match worker.await {
                        Ok(Err(code)) => { terminal.send(Err(fail(code))); }
                        Err(_) => { terminal.send(Err(fail(-9))); }
                        _ => {}
                    }
                }
                Ok(Err(e)) => {
                    terminal.send(Err(e));
                    let _ = worker.await;
                }
                Err(_) => {
                    terminal.send(Err(io::Error::other("input pump stopped")));
                    let _ = worker.await;
                }
            }
        }
        result = &mut worker => {
            pump.abort();
            match result {
                Ok(Err(code)) => { terminal.send(Err(fail(code))); }
                Err(_) => { terminal.send(Err(fail(-9))); }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests;

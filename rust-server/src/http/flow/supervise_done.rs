use super::Chunk;
use std::io;
use tokio::{sync::mpsc, task::JoinHandle};

pub async fn run(worker: JoinHandle<Result<(), i32>>, terminal: mpsc::OwnedPermit<Chunk>) {
    match worker.await {
        Ok(Err(code)) => {
            let _ = terminal.send(Err(io::Error::other(format!("VM error {code}"))));
        }
        Err(_) => {
            let _ = terminal.send(Err(io::Error::other("VM error -9")));
        }
        _ => {}
    }
}

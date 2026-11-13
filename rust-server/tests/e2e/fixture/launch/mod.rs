mod start;
use std::{path::PathBuf, process::Child};
pub struct Server {
    pub home: PathBuf,
    pub address: String,
    pub client: reqwest::Client,
    pub(super) child: Child,
}
impl Server {
    pub async fn new() -> Self {
        for _ in 0..10 {
            match Self::attempt().await {
                Ok(server) => return server,
                Err(message) if message.contains("Address already in use") => continue,
                Err(message) => panic!("server startup: {message}"),
            }
        }
        panic!("unable to reserve an isolated HTTP port");
    }
}

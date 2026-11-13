use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
pub struct Server {
    pub home: PathBuf,
    pub address: String,
    pub client: reqwest::Client,
    pub(super) child: Child,
}
impl Server {
    pub async fn new() -> Self {
        let home = std::env::temp_dir().join(format!("skrynia-e2e-{}", rand::random::<u64>()));
        fs::create_dir(&home).unwrap();
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        drop(socket);
        let child = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_DATA_DIR", &home)
            .env("SKRYNIA_PORT", port.to_string())
            .env("SKRYNIA_URL", format!("http://127.0.0.1:{port}/api"))
            .env("SKRYNIA_TEST_DISABLE_PUSH", "1")
            .stderr(Stdio::null())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let instance = Self {
            home,
            address: format!("http://127.0.0.1:{port}/api"),
            client: reqwest::Client::new(),
            child,
        };
        for _ in 0..100 {
            if instance
                .call(reqwest::Method::GET, "/health")
                .send()
                .await
                .is_ok()
            {
                return instance;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("server never became healthy");
    }
}

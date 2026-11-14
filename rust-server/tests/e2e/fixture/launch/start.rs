use super::Server;
use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
};
impl Server {
    pub async fn attempt() -> Result<Self, String> {
        let home = std::env::temp_dir().join(format!("skrynia-e2e-{}", rand::random::<u64>()));
        fs::create_dir(&home).unwrap();
        let socket = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = socket.local_addr().unwrap().port();
        drop(socket);
        let base = format!("http://127.0.0.1:{port}/t{}", rand::random::<u64>());
        let child = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_DATA_DIR", &home)
            .env("SKRYNIA_PORT", port.to_string())
            .env("SKRYNIA_URL", &base)
            .env("SKRYNIA_TEST_DISABLE_PUSH", "1")
            .env("SKRYNIA_PUSH_ALLOWED_HOSTS", "push.example.org")
            .stderr(fs::File::create(home.join("startup.log")).unwrap())
            .stdout(Stdio::null())
            .spawn()
            .unwrap();
        let mut instance = Self {
            home,
            address: base,
            client: reqwest::Client::new(),
            child,
        };
        for _ in 0..100 {
            if instance.child.try_wait().unwrap().is_some() {
                return Err(fs::read_to_string(instance.home.join("startup.log"))
                    .unwrap_or_else(|error| error.to_string()));
            }
            let response = instance.call(reqwest::Method::GET, "/health").send().await;
            if response.is_ok_and(|r| r.status().is_success()) {
                return Ok(instance);
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        Err("server never became healthy".into())
    }
}

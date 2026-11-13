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
        let base = format!("http://127.0.0.1:{port}/t{}", rand::random::<u64>());
        let child = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_DATA_DIR", &home)
            .env("SKRYNIA_PORT", port.to_string())
            .env("SKRYNIA_URL", &base)
            .env("SKRYNIA_TEST_DISABLE_PUSH", "1")
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
            assert!(
                instance.child.try_wait().unwrap().is_none(),
                "server exited during startup: {}",
                fs::read_to_string(instance.home.join("startup.log")).unwrap_or_default()
            );
            let reply = instance.call(reqwest::Method::GET, "/health").send().await;
            if reply.is_ok_and(|response| response.status().is_success()) {
                return instance;
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("server never became healthy");
    }
}

use super::Target;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Duration;
impl Target {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        let home = std::env::temp_dir().join(format!("skrynia-bench-{}", rand::random::<u64>()));
        fs::create_dir_all(&home)?;
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);
        let executable = std::env::current_exe()?
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("skrynia-server");
        let child = Command::new(executable)
            .env("SKRYNIA_DATA_DIR", &home)
            .env("SKRYNIA_URL", format!("http://127.0.0.1:{port}"))
            .env("SKRYNIA_PORT", port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        let target = Self {
            base: format!("http://127.0.0.1:{port}"),
            client: reqwest::Client::new(),
            home,
            child,
        };
        for _ in 0..100 {
            let result = target
                .client
                .get(format!("{}/health", target.base))
                .send()
                .await;
            if result.is_ok_and(|response| response.status().is_success()) {
                return Ok(target);
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        Err("benchmark server did not become ready".into())
    }
}

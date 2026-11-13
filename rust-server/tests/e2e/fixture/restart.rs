use super::Server;
use reqwest::Method;
use std::process::{Command, Stdio};
use std::time::Duration;
impl Server {
    pub async fn restart_after_kill(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        let port = url::Url::parse(&self.address).unwrap().port().unwrap();
        self.child = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_DATA_DIR", &self.home)
            .env("SKRYNIA_PORT", port.to_string())
            .env("SKRYNIA_URL", &self.address)
            .env("SKRYNIA_TEST_DISABLE_PUSH", "1")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        for _ in 0..100 {
            if let Ok(reply) = self.call(Method::GET, "/health").send().await {
                if reply.status().is_success() {
                    return;
                }
            }
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
        panic!("restarted server did not become healthy");
    }
}

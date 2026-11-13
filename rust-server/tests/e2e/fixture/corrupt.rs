use super::Server;
use serde_json::Value;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
pub fn inject(path: &Path, nested: Option<&str>) {
    let mut record: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    let target = match nested {
        Some("user") => &mut record["users"][0],
        Some("keys") => &mut record["keys"],
        _ => &mut record,
    };
    target["unexpected"] = Value::Bool(true);
    fs::write(path, serde_json::to_vec(&record).unwrap()).unwrap();
}
impl Server {
    pub async fn reject_restart(&mut self) {
        self.child.kill().unwrap();
        self.child.wait().unwrap();
        let mut child = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .env("SKRYNIA_DATA_DIR", &self.home)
            .env("SKRYNIA_URL", "http://127.0.0.1:17380")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        for _ in 0..100 {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(!status.success(), "corrupted record accepted");
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        child.kill().unwrap();
        child.wait().unwrap();
        panic!("corrupted state allowed server startup");
    }
}

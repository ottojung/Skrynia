use super::data_dir;
use std::{
    fs,
    process::{Command, Stdio},
    time::Duration,
};
#[tokio::test]
async fn permitted_maximum_key_length_allows_server_startup() {
    let home = data_dir();
    fs::create_dir(&home).unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let mut child = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
        .env("SKRYNIA_DATA_DIR", &home)
        .env("SKRYNIA_MAX_KEY_LENGTH", "255")
        .env("SKRYNIA_PORT", port.to_string())
        .env("SKRYNIA_URL", format!("http://127.0.0.1:{port}"))
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let client = reqwest::Client::new();
    let mut healthy = false;
    for _ in 0..100 {
        if child.try_wait().unwrap().is_some() {
            break;
        }
        let result = client
            .get(format!("http://127.0.0.1:{port}/health"))
            .send()
            .await;
        if result.is_ok_and(|r| r.status().is_success()) {
            healthy = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    if !healthy {
        child.kill().ok();
        let output = child.wait_with_output().unwrap();
        panic!(
            "valid key limit did not start: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    child.kill().unwrap();
    child.wait().unwrap();
    fs::remove_dir_all(home).unwrap();
}

use super::Server;
use reqwest::{Method, StatusCode};
use std::process::{Command, Stdio};
use std::time::Duration;
#[tokio::test]
async fn exclusive_writer_lock_prevents_a_second_process() {
    let server = Server::new().await;
    let mut second = Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
        .env("SKRYNIA_DATA_DIR", &server.home)
        .env("SKRYNIA_URL", "http://127.0.0.1:17382")
        .env("SKRYNIA_PORT", "17382")
        .stderr(Stdio::null())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let mut exit = None;
    for _ in 0..100 {
        if let Some(status) = second.try_wait().unwrap() {
            exit = Some(status);
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    if exit.is_none() {
        second.kill().unwrap();
        second.wait().unwrap();
    }
    assert!(
        exit.is_some_and(|status| !status.success()),
        "second writer became active"
    );
    assert_eq!(
        server
            .call(Method::GET, "/health")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
}

use super::Server;
use std::process::Command;
#[tokio::test]
async fn container_health_probe_recognizes_live_http_server() {
    let server = Server::new().await;
    let port = url::Url::parse(&server.address).unwrap().port().unwrap();
    let probe = |port: u16| {
        Command::new(env!("CARGO_BIN_EXE_skrynia-server"))
            .arg("healthcheck")
            .env("SKRYNIA_URL", &server.address)
            .env("SKRYNIA_PORT", port.to_string())
            .output()
            .unwrap()
            .status
            .success()
    };
    assert!(probe(port));
    assert!(!probe(0));
}

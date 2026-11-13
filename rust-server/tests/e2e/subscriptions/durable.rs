use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use std::fs;
#[tokio::test]
async fn pending_notifications_survive_restarts_and_coalesce() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let id = register(&server, &token, "https://push.example.org/one").await;
    let path = server
        .home
        .join("state/push/outbox")
        .join(format!("{id}.json"));
    assert_eq!(
        publish(&server, &token, Method::POST, "first").await,
        StatusCode::CREATED
    );
    let first: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        publish(&server, &token, Method::PUT, "next").await,
        StatusCode::OK
    );
    let next: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert!(next["generation"].as_u64() > first["generation"].as_u64());
    server.restart_after_kill().await;
    assert!(path.exists());
    assert_eq!(
        fs::read_dir(server.home.join("state/push/outbox"))
            .unwrap()
            .count(),
        1
    );
}

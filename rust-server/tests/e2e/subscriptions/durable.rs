use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use serde_json::Value;
use std::fs;
#[tokio::test]
async fn pending_notifications_survive_restarts_and_coalesce() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let id = register(&server, &token, "https://push.example.org/one").await;
    let path = server.home.join("state/push/outbox.json");
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
    assert!(next[&id]["generation"].as_u64() > first[&id]["generation"].as_u64());
    server.restart_after_kill().await;
    assert!(path.exists());
    let loaded: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(loaded.as_object().unwrap().len(), 1);
    assert_eq!(loaded[&id], next[&id]);
}

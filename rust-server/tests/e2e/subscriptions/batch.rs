use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn grouped_notifications_survive_two_restarts_with_monotone_generations() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let first = register(&server, &token, "https://push.example.org/one").await;
    let second = register(&server, &token, "https://push.example.org/two").await;
    let path = server.home.join("state/push/outbox.json");
    for (method, payload, generation) in [
        (Method::POST, "first", 1u64),
        (Method::PUT, "second", 2),
        (Method::PUT, "third", 3),
    ] {
        let status = publish(&server, &token, method, payload).await;
        assert!(matches!(status, StatusCode::CREATED | StatusCode::OK));
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved[&first]["generation"], generation);
        assert_eq!(saved[&second]["generation"], generation);
        server.restart_after_kill().await;
    }
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(saved.as_object().unwrap().len(), 2);
}

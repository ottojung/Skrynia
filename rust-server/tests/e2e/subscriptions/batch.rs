use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn grouped_notifications_survive_two_restarts_with_monotone_generations() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let first = register(&server, &token, "https://push.example.org/one").await;
    let second = register(&server, &token, "https://push.example.org/two").await;
    let index = server.home.join("state/push/outbox-generations.json");
    for (method, payload, generation) in [
        (Method::POST, "first", 1u64),
        (Method::PUT, "second", 2),
        (Method::PUT, "third", 3),
    ] {
        let status = publish(&server, &token, method, payload).await;
        assert!(matches!(status, StatusCode::CREATED | StatusCode::OK));
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&index).unwrap()).unwrap();
        assert_eq!(saved[&first], generation);
        assert_eq!(saved[&second], generation);
        server.restart_after_kill().await;
    }
    assert_eq!(
        fs::read_dir(server.home.join("state/push/outbox"))
            .unwrap()
            .count(),
        0
    );
}

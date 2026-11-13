use super::{Server, register};
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn failed_group_persistence_keeps_committed_notification_replayable() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let ids = [
        register(&server, &token, "https://push.example.org/one").await,
        register(&server, &token, "https://push.example.org/two").await,
    ];
    let path = server.home.join("state/push/outbox.json");
    let moved = server.home.join("state/push/temporary");
    fs::rename(&path, &moved).unwrap();
    fs::create_dir(&path).unwrap();
    let response = server
        .call(Method::POST, "/store/notice.retry")
        .bearer_auth(&token)
        .body("value")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["notificationPending"], true);
    let journal = server.home.join("state/store-transactions");
    assert_eq!(fs::read_dir(&journal).unwrap().count(), 1);
    fs::remove_dir(&path).unwrap();
    fs::rename(moved, &path).unwrap();
    server.restart_after_kill().await;
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    for id in ids {
        assert_eq!(saved[&id]["generation"], 1);
    }
    assert_eq!(fs::read_dir(journal).unwrap().count(), 0);
}

use super::{Server, publish};
use crate::e2e::fixture::subscription;
use reqwest::{Method, StatusCode};
use serde_json::json;
use std::fs;

#[tokio::test]
async fn unrelated_keys_do_not_enqueue_notifications() {
    let server = Server::new().await;
    let token = server.user().await;
    let mut payload = subscription("https://push.example.org/exact");
    payload["locations"] = json!("notice.event");
    let response = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(
        server
            .call(Method::POST, "/store/other.event")
            .bearer_auth(&token)
            .body("x")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CREATED
    );
    let path = server.home.join("state/push/outbox.json");
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert!(saved.as_object().unwrap().is_empty());
    assert_eq!(
        publish(&server, &token, Method::POST, "hit").await,
        StatusCode::CREATED
    );
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(saved.as_object().unwrap().len(), 1);
}

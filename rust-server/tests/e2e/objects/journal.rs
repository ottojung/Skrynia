use super::Server;
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn intent_directory_failure_does_not_disable_healthy_requests() {
    let server = Server::new().await;
    let token = server.user().await;
    let dir = server.home.join("state/store-transactions");
    let moved = server.home.join("state/transactions-moved");
    fs::rename(&dir, &moved).unwrap();
    fs::write(&dir, b"not a directory").unwrap();
    let result = server
        .call(Method::POST, "/store/intent_retry")
        .bearer_auth(&token)
        .body("payload")
        .send()
        .await
        .unwrap();
    assert!(
        matches!(
            result.status(),
            StatusCode::SERVICE_UNAVAILABLE | StatusCode::INSUFFICIENT_STORAGE
        ),
        "unexpected: {}",
        result.status()
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
    fs::remove_file(&dir).unwrap();
    fs::rename(&moved, &dir).unwrap();
    let retry = server
        .call(Method::POST, "/store/intent_retry")
        .bearer_auth(&token)
        .body("payload")
        .send()
        .await
        .unwrap();
    assert_eq!(retry.status(), StatusCode::CREATED);
}

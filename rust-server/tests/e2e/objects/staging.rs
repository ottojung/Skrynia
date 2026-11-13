use super::Server;
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn failed_staging_is_retryable_without_disabling_public_reads() {
    let server = Server::new().await;
    let token = server.user().await;
    let staging = server.home.join("store-tmp");
    let moved = server.home.join("staging-moved");
    fs::rename(&staging, &moved).unwrap();
    fs::write(&staging, b"unavailable").unwrap();
    let response = server
        .call(Method::POST, "/store/stage_failure")
        .bearer_auth(&token)
        .body("value")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        server
            .call(Method::GET, "/health")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    fs::remove_file(&staging).unwrap();
    fs::rename(&moved, &staging).unwrap();
    assert_eq!(
        server
            .call(Method::POST, "/store/stage_failure")
            .bearer_auth(token)
            .body("value")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CREATED
    );
}

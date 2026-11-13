use super::Server;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn key_limit_respects_filesystem_and_keeps_server_healthy() {
    let server = Server::new().await;
    let token = server.user().await;
    let good = format!("/store/{}", "a".repeat(255));
    let created = server
        .call(Method::POST, &good)
        .bearer_auth(&token)
        .body("valid")
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let bad = format!("/store/{}", "b".repeat(256));
    let result = server
        .call(Method::POST, &bad)
        .bearer_auth(&token)
        .body("invalid")
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::BAD_REQUEST);
    assert_eq!(
        result.json::<serde_json::Value>().await.unwrap()["error"],
        "invalid_key"
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

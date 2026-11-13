use super::fixture::Server;
use reqwest::{Method, StatusCode};
use serde_json::json;
#[tokio::test]
async fn push_registration_rejects_private_targets() {
    let server = Server::new().await;
    let token = server.user().await;
    let invalid = json!({"locations":"*","events":["create"],
        "endpoint":"https://127.0.0.1/push",
        "keys":{"p256dh":"invalid","auth":"invalid"}});
    let result = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&invalid)
        .send()
        .await
        .unwrap();
    assert_ne!(result.status(), StatusCode::CREATED);
}

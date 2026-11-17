use super::Server;
use crate::e2e::fixture::subscription;
use reqwest::{Method, StatusCode};
use serde_json::json;

async fn register_payload(
    server: &Server,
    token: &str,
    payload: serde_json::Value,
) -> reqwest::Response {
    server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&payload)
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn rejects_prefix_wildcard_and_empty_locations() {
    let server = Server::new().await;
    let token = server.user().await;
    for locations in ["*", "notice.*", "notice/", "", "apps/long/*"] {
        let mut payload = subscription("https://push.example.org/reject");
        payload["locations"] = json!(locations);
        let response = register_payload(&server, &token, payload).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST, "{locations}");
    }
}

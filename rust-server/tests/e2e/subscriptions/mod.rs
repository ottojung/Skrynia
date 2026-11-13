use super::fixture::{Server, subscription};
use reqwest::{Method, StatusCode};
use serde_json::Value;
mod durable;
mod invalid;
mod revoked;
async fn register(server: &Server, token: &str, endpoint: &str) -> String {
    let result = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&subscription(endpoint))
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::CREATED);
    let data: Value = result.json().await.unwrap();
    data["id"].as_str().unwrap().into()
}
async fn publish(server: &Server, token: &str, method: Method, data: &str) -> StatusCode {
    server
        .call(method, "/store/notice.event")
        .bearer_auth(token)
        .body(data.to_owned())
        .send()
        .await
        .unwrap()
        .status()
}

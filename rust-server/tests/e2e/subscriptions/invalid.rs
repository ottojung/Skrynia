use super::{Server, subscription};
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn push_registration_rejects_private_targets() {
    let server = Server::new().await;
    let token = server.user().await;
    let result = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&subscription("https://127.0.0.1/push"))
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::BAD_REQUEST);
}

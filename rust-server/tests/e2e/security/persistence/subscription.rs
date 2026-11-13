use crate::e2e::fixture::{Server, corrupt::inject, subscription};
use reqwest::Method;
pub(super) async fn setup() -> (Server, String) {
    let server = Server::new().await;
    let token = server.user().await;
    let sub = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&subscription("https://push.example.org/example"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    (server, sub["id"].as_str().unwrap().into())
}
#[tokio::test]
async fn unknown_subscription_fields_fail_startup() {
    let (mut server, id) = setup().await;
    inject(
        &server.home.join(format!("state/push-subs/{id}.json")),
        None,
    );
    server.reject_restart().await;
}
#[tokio::test]
async fn unknown_push_keys_fields_fail_startup() {
    let (mut server, id) = setup().await;
    inject(
        &server.home.join(format!("state/push-subs/{id}.json")),
        Some("keys"),
    );
    server.reject_restart().await;
}

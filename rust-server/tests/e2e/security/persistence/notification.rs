use super::subscription::setup;
use crate::e2e::fixture::corrupt::inject;
use reqwest::Method;
#[tokio::test]
async fn unknown_notification_fields_fail_startup() {
    let (mut server, id) = setup().await;
    let token = server.user().await;
    let reply = server
        .call(Method::POST, "/store/notice.first")
        .bearer_auth(token)
        .body("value")
        .send()
        .await
        .unwrap();
    assert!(reply.status().is_success());
    inject(
        &server.home.join(format!("state/push/outbox/{id}.json")),
        None,
    );
    server.reject_restart().await;
}

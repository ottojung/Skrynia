use super::subscription::setup;
use reqwest::Method;
use std::fs;
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
    let path = server.home.join("state/push/outbox.json");
    let mut record: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    record[&id]["unexpected"] = true.into();
    fs::write(path, serde_json::to_vec(&record).unwrap()).unwrap();
    server.reject_restart().await;
}

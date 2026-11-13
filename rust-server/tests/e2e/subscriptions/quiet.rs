use super::{Server, subscription};
use reqwest::{Method, StatusCode};
use std::{fs, os::unix::fs::MetadataExt};
#[tokio::test]
async fn deleting_idle_subscription_does_not_rewrite_outbox() {
    let server = Server::new().await;
    let token = server.user().await;
    let response = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&subscription("https://push.example.org/quiet"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let created: serde_json::Value = response.json().await.unwrap();
    let id = created["id"].as_str().unwrap();
    let cap = created["capability"].as_str().unwrap();
    let file = server.home.join("state/push/outbox.json");
    let before = fs::metadata(&file).unwrap().ino();
    let response = server
        .call(Method::DELETE, &format!("/push/subscriptions/{id}"))
        .header("x-skrynia-capability", cap)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        fs::metadata(&file).unwrap().ino(),
        before,
        "unnecessary outbox fsync"
    );
    assert!(
        !server
            .home
            .join(format!("state/push-subs/{id}.json"))
            .exists()
    );
}

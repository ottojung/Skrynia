use crate::e2e::fixture::{Server, subscription};
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn legacy_invalid_push_keys_do_not_prevent_startup_or_repair() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let input = subscription("https://push.example.org/legacy");
    let response = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(&token)
        .json(&input)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    let registered: serde_json::Value = response.json().await.unwrap();
    let id = registered["id"].as_str().unwrap();
    let capability = registered["capability"].as_str().unwrap();
    server
        .restart_with(|home| {
            let path = home.join(format!("state/push-subs/{id}.json"));
            let mut stored: serde_json::Value =
                serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
            stored["keys"]["p256dh"] = "AAAA".into();
            fs::write(&path, serde_json::to_vec(&stored).unwrap()).unwrap();
        })
        .await;
    let result = server
        .call(Method::PUT, &format!("/push/subscriptions/{id}"))
        .header("X-Skrynia-Capability", capability)
        .json(&serde_json::json!({"keys":input["keys"]}))
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    server.restart_after_kill().await;
}

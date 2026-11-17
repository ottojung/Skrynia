use super::Server;
use crate::e2e::fixture::subscription;
use reqwest::{Method, StatusCode};
use serde_json::json;

#[tokio::test]
async fn prefix_grant_can_subscribe_to_exact_key_in_scope() {
    let server = Server::new().await;
    let root = server.user().await;
    let body = json!({"locations":"apps/long/*","operations":["subscribe"],"maxBytes":1000000,"maxObjects":100});
    let create = server
        .call(Method::POST, "/grants")
        .bearer_auth(&root)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), StatusCode::OK);
    let grant: serde_json::Value = create.json().await.unwrap();
    let path = format!("/grants/{}/users", grant["id"].as_str().unwrap());
    let issue = server
        .call(Method::POST, &path)
        .bearer_auth(grant["issuerToken"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(issue.status(), StatusCode::OK);
    let issued: serde_json::Value = issue.json().await.unwrap();
    let token = issued["userToken"].as_str().unwrap();
    let mut in_scope = subscription("https://push.example.org/scoped");
    in_scope["locations"] = json!("apps/long/releases/v1/assets/app.js");
    let ok = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&in_scope)
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), StatusCode::CREATED);
    let mut out = subscription("https://push.example.org/outside");
    out["locations"] = json!("other/file.txt");
    let bad = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&out)
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status(), StatusCode::FORBIDDEN);
}

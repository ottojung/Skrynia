use super::Server;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
pub async fn create(server: &Server, parent: &str, scope: &str, max: u64) -> Value {
    let response = server
        .call(Method::POST, "/grants")
        .bearer_auth(parent)
        .json(
            &json!({"locations":scope, "operations":["create","modify","delete","delegate","list"],
            "maxBytes":max, "maxObjects":10}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response.json().await.unwrap()
}
pub async fn issue(server: &Server, grant: &Value) -> Value {
    let id = grant["id"].as_str().unwrap();
    let token = grant["issuerToken"].as_str().unwrap();
    let response = server
        .call(Method::POST, &format!("/grants/{id}/users"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    response.json().await.unwrap()
}

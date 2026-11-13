use super::Server;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
pub async fn grant(server: &Server, parent: &str, scope: &str, limit: u64) -> (String, String) {
    let result = server
        .call(Method::POST, "/grants")
        .bearer_auth(parent)
        .json(
            &json!({"locations":scope,"operations":["create","modify","delete","list","delegate"],
            "maxBytes":limit,"maxObjects":10}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    let value: Value = result.json().await.unwrap();
    let id = value["id"].as_str().unwrap().to_owned();
    let issued = server
        .call(Method::POST, &format!("/grants/{id}/users"))
        .bearer_auth(value["issuerToken"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(issued.status(), StatusCode::OK);
    let token: Value = issued.json().await.unwrap();
    (id, token["userToken"].as_str().unwrap().to_owned())
}
pub async fn usage(server: &Server, id: &str, token: &str) -> Value {
    let reply = server
        .call(Method::GET, &format!("/grants/{id}"))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), StatusCode::OK);
    let value: Value = reply.json().await.unwrap();
    value["usage"].clone()
}

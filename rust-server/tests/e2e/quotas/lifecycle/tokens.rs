use super::{Method, Server, StatusCode, issue, usage};
use serde_json::Value;
pub async fn cycle(server: &Server, parent: &Value, id: &str, token: &str) {
    let extra = issue(server, parent).await;
    assert_eq!(usage(server, id, token).await["bytes"], 4224);
    let user_id = extra["userId"].as_str().unwrap();
    let reply = server
        .call(Method::DELETE, &format!("/grants/{id}/users/{user_id}"))
        .bearer_auth(parent["issuerToken"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), StatusCode::OK);
    assert_eq!(usage(server, id, token).await["bytes"], 4096);
}

use super::{Method, Server, StatusCode, send, usage};
use serde_json::Value;
pub async fn exercise(
    server: &mut Server,
    id: &str,
    token: &str,
    grand: &Value,
    grand_token: &str,
) {
    assert_eq!(
        send(
            server,
            Method::POST,
            "team.abc",
            grand_token,
            &"x".repeat(100)
        )
        .await,
        StatusCode::CREATED
    );
    assert_eq!(usage(server, id, token).await["bytes"], 4708);
    assert_eq!(
        send(
            server,
            Method::PUT,
            "team.abc",
            grand_token,
            &"y".repeat(600)
        )
        .await,
        StatusCode::INSUFFICIENT_STORAGE
    );
    let grand_id = grand["id"].as_str().unwrap();
    let reply = server
        .call(Method::DELETE, &format!("/grants/{grand_id}"))
        .bearer_auth(grand["issuerToken"].as_str().unwrap())
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), StatusCode::OK);
    assert_eq!(usage(server, id, token).await["bytes"], 2660);
    server.restart_after_kill().await;
    assert_eq!(usage(server, id, token).await["bytes"], 2660);
}

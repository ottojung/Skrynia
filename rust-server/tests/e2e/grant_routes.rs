use super::fixture::Server;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn trailing_slash_does_not_panic_or_create_grant() {
    let server = Server::new().await;
    let user = server.user().await;
    let reply = server
        .call(Method::POST, "/grants/")
        .bearer_auth(user)
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        reply.json::<serde_json::Value>().await.unwrap()["error"],
        "not_found"
    );
    assert_eq!(
        server
            .call(Method::GET, "/health")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
}

use super::fixture::Server;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn delegation_is_limited_to_granted_scope() {
    let server = Server::new().await;
    let root = server.operator_credentials();
    let (id, issuer) = server.root();
    assert_eq!(root["id"], id);
    assert_eq!(root["issuerToken"], issuer);
    let user = server.user().await;
    let token = server.delegated_user(&user).await;
    let good = server
        .call(Method::POST, "/store/team.first")
        .bearer_auth(&token)
        .body("yes")
        .send()
        .await
        .unwrap();
    assert_eq!(good.status(), StatusCode::CREATED);
    let denied = server
        .call(Method::POST, "/store/other.first")
        .bearer_auth(&token)
        .body("no")
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
    let denied = server
        .call(Method::PUT, "/store/team.first")
        .bearer_auth(&token)
        .body("no")
        .send()
        .await
        .unwrap();
    assert_eq!(denied.status(), StatusCode::FORBIDDEN);
}

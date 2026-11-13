use super::Server;
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn hidden_state_never_exposes_grant_secrets() {
    let server = Server::new().await;
    let (_, issuer) = server.root();
    let result = server
        .call(Method::GET, "/state/grants")
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::NOT_FOUND);
    assert!(!result.text().await.unwrap().contains(&issuer));
    assert!(
        fs::read_dir(server.home.join("state/store-transactions"))
            .unwrap()
            .next()
            .is_none()
    );
}

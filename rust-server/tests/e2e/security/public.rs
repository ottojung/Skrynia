use super::Server;
use reqwest::{Method, StatusCode};
use std::fs;
use std::os::unix::fs::symlink;
#[tokio::test]
async fn public_symlinks_cannot_expose_private_grant_records() {
    let server = Server::new().await;
    let (id, issuer) = server.root();
    let secret = server.home.join(format!("state/grants/{id}.json"));
    let link = server.home.join("store/disclosure");
    symlink(&secret, &link).unwrap();
    let response = server
        .call(Method::GET, "/store/disclosure")
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = response.text().await.unwrap();
    assert!(!body.contains(&issuer));
    assert_eq!(
        server
            .call(Method::GET, "/health")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    fs::remove_file(&link).unwrap();
}

use super::Server;
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn acknowledged_bytes_and_grant_authority_survive_process_kill() {
    let mut server = Server::new().await;
    let token = server.user().await;
    let path = "/store/durable.data";
    let bytes = vec![0x2au8; 256 * 1024];
    assert_eq!(
        server
            .call(Method::POST, path)
            .bearer_auth(&token)
            .body(bytes.clone())
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::CREATED
    );
    server.restart_after_kill().await;
    let response = server.call(Method::GET, path).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.bytes().await.unwrap(), bytes);
    assert_eq!(
        server
            .call(Method::DELETE, path)
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    assert!(
        fs::read_dir(server.home.join("state/store-transactions"))
            .unwrap()
            .next()
            .is_none()
    );
}

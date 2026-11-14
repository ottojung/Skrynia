use super::{Server, subscription};
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn push_registration_rejects_private_targets() {
    let server = Server::new().await;
    let token = server.user().await;
    let result = server
        .call(Method::POST, "/push/subscriptions")
        .bearer_auth(token)
        .json(&subscription("https://127.0.0.1/push"))
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::BAD_REQUEST);
}
#[tokio::test]
async fn public_https_push_hosts_need_no_allowlist() {
    let mut server = Server::new().await;
    let token = server.user().await;
    for host in ["push.example.org", "other-push.example.net"] {
        let endpoint = format!("https://{host}/subscription");
        let response = server
            .call(Method::POST, "/push/subscriptions")
            .bearer_auth(&token)
            .json(&subscription(&endpoint))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED, "{host}");
    }
    server.restart_after_kill().await;
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

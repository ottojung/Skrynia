mod extra;
use super::Server;
use super::common::{grant, usage};
use super::requests::send;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn nested_files_are_public_and_grant_quotas_survive_restart() {
    let mut server = Server::new().await;
    let root = server.user().await;
    let key = "apps/long/releases/v1/assets/index-A1.js";
    assert_eq!(
        send(&server, Method::POST, key, &root, "hello").await,
        StatusCode::CREATED
    );
    let (id, child) = grant(&server, &root, "apps/long/*", 10_000).await;
    assert_eq!(usage(&server, &id, &child).await["objects"], 1);
    assert_eq!(
        send(&server, Method::POST, "apps/long/x.css", &child, "a").await,
        StatusCode::CREATED
    );
    assert_eq!(
        send(&server, Method::POST, "apps/other/x.css", &child, "a").await,
        StatusCode::FORBIDDEN
    );
    let response = server
        .call(Method::GET, &format!("/store/{key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers()["content-type"],
        "text/javascript; charset=utf-8"
    );
    assert_eq!(response.text().await.unwrap(), "hello");
    extra::check(&server, &root, &child).await;
    server.restart_after_kill().await;
    assert_eq!(usage(&server, &id, &child).await["objects"], 2);
    assert_eq!(
        send(&server, Method::PUT, key, &root, "world").await,
        StatusCode::OK
    );
    assert_eq!(
        send(&server, Method::DELETE, key, &root, "").await,
        StatusCode::OK
    );
    assert_eq!(usage(&server, &id, &child).await["objects"], 1);
}

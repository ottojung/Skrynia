use super::Server;
use super::common::{grant, usage};
use super::requests::send;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn scopes_include_old_objects_and_unrelated_mutations() {
    let mut server = Server::new().await;
    let root = server.user().await;
    let first = "x".repeat(100);
    assert_eq!(
        send(&server, Method::POST, "team.one", &root, &first).await,
        StatusCode::CREATED
    );
    let (id, child) = grant(&server, &root, "team.*", 2680).await;
    assert_eq!(usage(&server, &id, &child).await["bytes"], 2660);
    assert_eq!(
        send(&server, Method::POST, "team.two", &child, "x").await,
        StatusCode::INSUFFICIENT_STORAGE
    );
    let bigger = "y".repeat(200);
    assert_eq!(
        send(&server, Method::PUT, "team.one", &root, &bigger).await,
        StatusCode::OK
    );
    assert_eq!(usage(&server, &id, &child).await["bytes"], 2760);
    assert_eq!(
        send(&server, Method::PUT, "team.one", &child, "x").await,
        StatusCode::OK
    );
    assert_eq!(usage(&server, &id, &child).await["bytes"], 2561);
    assert_eq!(
        send(&server, Method::POST, "team.two", &root, "yy").await,
        StatusCode::CREATED
    );
    assert_eq!(usage(&server, &id, &child).await["bytes"], 3075);
    server.restart_after_kill().await;
    assert_eq!(usage(&server, &id, &child).await["bytes"], 3075);
    assert_eq!(
        send(&server, Method::DELETE, "team.two", &root, "").await,
        StatusCode::OK
    );
    assert_eq!(usage(&server, &id, &child).await["bytes"], 2561);
}

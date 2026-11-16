use super::Server;
use super::common::{grant, usage};
use super::requests::send;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn new_prefix_uses_flushed_index_and_restart_rebuilds_it() {
    let mut server = Server::new().await;
    let root = server.user().await;
    for i in 0..70 {
        let key = format!("idx.item{i:03}");
        assert_eq!(
            send(&server, Method::POST, &key, &root, "a").await,
            StatusCode::CREATED
        );
    }
    let (id, child) = grant(&server, &root, "idx.*", 100_000).await;
    let initial = usage(&server, &id, &child).await;
    assert_eq!(initial["bytes"], 2048 + 70 * 513);
    assert_eq!(initial["objects"], 70);
    assert_eq!(
        send(&server, Method::PUT, "idx.item000", &root, "abc").await,
        StatusCode::OK
    );
    assert_eq!(
        usage(&server, &id, &child).await["bytes"],
        2048 + 70 * 513 + 2
    );
    server.restart_after_kill().await;
    assert_eq!(
        usage(&server, &id, &child).await["bytes"],
        2048 + 70 * 513 + 2
    );
}

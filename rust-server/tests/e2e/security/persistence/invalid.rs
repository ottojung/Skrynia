use crate::e2e::fixture::Server;
use std::fs;
#[tokio::test]
async fn unknown_outbox_keys_reject_startup() {
    let mut server = Server::new().await;
    let path = server.home.join("state/push/outbox.json");
    fs::write(
        path,
        br#"{"malformed-id":{"generation":2,"attempts":0,"nextAt":0}}"#,
    )
    .unwrap();
    server.reject_restart().await;
}
#[tokio::test]
async fn zero_outbox_generation_rejects_startup() {
    let mut server = Server::new().await;
    let path = server.home.join("state/push/outbox.json");
    let id = "a".repeat(32);
    let value = serde_json::json!({id:{"generation":0,"attempts":0,"nextAt":0}});
    fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
    server.reject_restart().await;
}
#[tokio::test]
async fn symlinked_outbox_rejects_startup() {
    let mut server = Server::new().await;
    let (root, _) = server.root();
    let secret = server.home.join(format!("state/grants/{root}.json"));
    let index = server.home.join("state/push/outbox.json");
    fs::remove_file(&index).unwrap();
    std::os::unix::fs::symlink(secret, index).unwrap();
    server.reject_restart().await;
}

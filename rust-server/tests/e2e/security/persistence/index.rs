use crate::e2e::fixture::Server;
use std::fs;
#[tokio::test]
async fn unknown_persisted_notification_generations_reject_startup() {
    let mut server = Server::new().await;
    let path = server.home.join("state/push/outbox-generations.json");
    fs::write(path, br#"{"malformed-id":2}"#).unwrap();
    server.reject_restart().await;
}
#[tokio::test]
async fn zero_persisted_notification_generations_reject_startup() {
    let mut server = Server::new().await;
    let path = server.home.join("state/push/outbox-generations.json");
    fs::write(path, format!(r#"{{"{}":0}}"#, "a".repeat(32))).unwrap();
    server.reject_restart().await;
}

#[tokio::test]
async fn symlinked_generation_index_rejects_startup() {
    let mut server = Server::new().await;
    let (root, _) = server.root();
    let secret = server.home.join(format!("state/grants/{root}.json"));
    let index = server.home.join("state/push/outbox-generations.json");
    std::os::unix::fs::symlink(secret, index).unwrap();
    server.reject_restart().await;
}

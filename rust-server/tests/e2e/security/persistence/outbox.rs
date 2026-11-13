use crate::e2e::fixture::Server;
use std::fs;
#[tokio::test]
async fn orphaned_outbox_record_rejects_server_startup() {
    let mut server = Server::new().await;
    let missing = "a".repeat(32);
    let record = serde_json::json!({missing:{"generation":1,"attempts":0,"nextAt":0}});
    fs::write(
        server.home.join("state/push/outbox.json"),
        serde_json::to_vec(&record).unwrap(),
    )
    .unwrap();
    server.reject_restart().await;
}

use crate::e2e::fixture::{Server, corrupt::inject};
use std::fs;
#[tokio::test]
async fn unknown_intent_fields_fail_startup() {
    let mut server = Server::new().await;
    let record = server
        .home
        .join(format!("state/store-transactions/{}.json", "a".repeat(32)));
    fs::write(
        &record,
        br#"{"key":"first","kind":"delete","committed":false}"#,
    )
    .unwrap();
    inject(&record, None);
    server.reject_restart().await;
}
#[tokio::test]
async fn unknown_revocation_fields_fail_startup() {
    let mut server = Server::new().await;
    let (id, _) = server.root();
    let record = server
        .home
        .join(format!("state/grant-revocations/{id}.json"));
    fs::write(&record, format!(r#"{{"id":"{id}","ids":["{id}"]}}"#)).unwrap();
    inject(&record, None);
    server.reject_restart().await;
}
#[tokio::test]
async fn missing_committed_flag_fails_startup() {
    let mut server = Server::new().await;
    let record = server
        .home
        .join(format!("state/store-transactions/{}.json", "a".repeat(32)));
    fs::write(&record, br#"{"key":"first","kind":"delete"}"#).unwrap();
    server.reject_restart().await;
}

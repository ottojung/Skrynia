use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn grant_revocation_recovery_clears_pending_outbox_before_subscriptions() {
    let mut server = Server::new().await;
    let (id, _) = server.root();
    let token = server.user().await;
    let sub = register(&server, &token, "https://push.example.org/revoked").await;
    assert_eq!(
        publish(&server, &token, Method::POST, "pending").await,
        StatusCode::CREATED
    );
    server
        .restart_with(|root| {
            let marker = root.join(format!("state/grant-revocations/{id}.json"));
            let data = serde_json::json!({"id":id,"ids":[id]});
            fs::write(marker, serde_json::to_vec(&data).unwrap()).unwrap();
        })
        .await;
    assert!(
        !server
            .home
            .join(format!("state/push-subs/{sub}.json"))
            .exists()
    );
    let path = server.home.join("state/push/outbox.json");
    let pending: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert!(pending.as_object().unwrap().is_empty());
}

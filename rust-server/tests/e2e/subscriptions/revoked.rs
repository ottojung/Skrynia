use super::{Server, publish, register};
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn root_revocation_rotates_authority_without_deleting_bytes() {
    let mut server = Server::new().await;
    let (root, issuer) = server.root();
    let token = server.user().await;
    let id = register(&server, &token, "https://push.example.org/revoked").await;
    assert_eq!(
        publish(&server, &token, Method::POST, "survives").await,
        StatusCode::CREATED
    );
    let response = server
        .call(Method::DELETE, &format!("/grants/{root}"))
        .bearer_auth(issuer)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    server.restart_after_kill().await;
    assert_ne!(server.root().0, root);
    let contents = server
        .call(Method::GET, "/store/notice.event")
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(contents, "survives");
    assert!(
        !server
            .home
            .join(format!("state/push-subs/{id}.json"))
            .exists()
    );
    let records: serde_json::Value =
        serde_json::from_slice(&fs::read(server.home.join("state/push/outbox.json")).unwrap())
            .unwrap();
    assert!(records.as_object().unwrap().is_empty());
}

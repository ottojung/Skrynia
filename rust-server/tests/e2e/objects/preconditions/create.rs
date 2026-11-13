use super::Server;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn creation_and_modification_reject_bad_preconditions() {
    let server = Server::new().await;
    let user = server.user().await;
    let path = "/store/preconditions";
    let invalid = server
        .call(Method::POST, path)
        .bearer_auth(&user)
        .header("X-Skrynia-If-SHA256", "f".repeat(64))
        .body("ignored")
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let created = server
        .call(Method::POST, path)
        .bearer_auth(&user)
        .body("original")
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let invalid = server
        .call(Method::PUT, path)
        .bearer_auth(&user)
        .header("X-Skrynia-If-SHA256", "INVALID")
        .body("ignored")
        .send()
        .await
        .unwrap();
    assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    let text = server
        .call(Method::GET, path)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(text, "original");
}

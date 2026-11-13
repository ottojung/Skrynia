use super::Server;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn conditional_deletion_requires_current_digest() {
    let server = Server::new().await;
    let user = server.user().await;
    let path = "/store/preconditions";
    let created = server
        .call(Method::POST, path)
        .bearer_auth(&user)
        .body("original")
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let unsupported = server
        .call(Method::DELETE, path)
        .bearer_auth(&user)
        .header("If-Match", "*")
        .send()
        .await
        .unwrap();
    assert_eq!(unsupported.status(), StatusCode::BAD_REQUEST);
    let wrong = server
        .call(Method::DELETE, path)
        .bearer_auth(&user)
        .header("X-Skrynia-If-SHA256", "f".repeat(64))
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::PRECONDITION_FAILED);
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

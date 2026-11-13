use super::Server;
use reqwest::{Method, StatusCode};
use sha2::{Digest, Sha256};
#[tokio::test]
async fn stored_bytes_and_conditional_mutations() {
    let server = Server::new().await;
    let user = server.user().await;
    let path = "/store/example.dat";
    let written = server
        .call(Method::POST, path)
        .bearer_auth(&user)
        .body("hello")
        .send()
        .await
        .unwrap();
    assert_eq!(written.status(), StatusCode::CREATED);
    let read = server.call(Method::GET, path).send().await.unwrap();
    assert_eq!(read.status(), StatusCode::OK);
    let tag = read.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(read.bytes().await.unwrap(), "hello");
    let wrong = server
        .call(Method::PUT, path)
        .bearer_auth(&user)
        .body("bad")
        .header("X-Skrynia-If-SHA256", "0".repeat(64))
        .send()
        .await
        .unwrap();
    assert_eq!(wrong.status(), StatusCode::PRECONDITION_FAILED);
    let hash = hex::encode(Sha256::digest(b"hello"));
    assert_eq!(tag, format!("\"{hash}\""));
    let updated = server
        .call(Method::PUT, path)
        .bearer_auth(&user)
        .body("new")
        .header("X-Skrynia-If-SHA256", hash)
        .send()
        .await
        .unwrap();
    assert_eq!(updated.status(), StatusCode::OK);
    let response = server.call(Method::GET, path).send().await.unwrap();
    assert_eq!(response.text().await.unwrap(), "new");
}

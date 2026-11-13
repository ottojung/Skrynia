use super::Server;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn conditional_and_ranged_public_reads() {
    let server = Server::new().await;
    let token = server.user().await;
    let path = "/store/ranges.bin";
    let created = server
        .call(Method::POST, path)
        .bearer_auth(&token)
        .body("0123456789")
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    let partial = server
        .call(Method::GET, path)
        .header("Range", "bytes=2-4")
        .send()
        .await
        .unwrap();
    assert_eq!(partial.status(), StatusCode::PARTIAL_CONTENT);
    assert_eq!(partial.headers()["content-range"], "bytes 2-4/10");
    assert_eq!(partial.bytes().await.unwrap(), "234");
    let head = server.call(Method::HEAD, path).send().await.unwrap();
    assert_eq!(head.status(), StatusCode::OK);
    let tag = head.headers()["etag"].to_str().unwrap().to_owned();
    assert_eq!(head.headers()["content-length"], "10");
    assert!(head.bytes().await.unwrap().is_empty());
    assert_eq!(
        server
            .call(Method::GET, path)
            .header("If-None-Match", &tag)
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::NOT_MODIFIED
    );
    let plain = server
        .call(Method::GET, path)
        .header("Range", "bytes=0-0")
        .header("If-Range", "\"stale\"")
        .send()
        .await
        .unwrap();
    assert_eq!(plain.status(), StatusCode::OK);
    assert_eq!(plain.bytes().await.unwrap(), "0123456789");
}

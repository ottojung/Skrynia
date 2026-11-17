use super::Server;
use futures_util::{StreamExt, stream};
use reqwest::{Method, StatusCode};
const BLOCK: usize = 1_048_576;
const BLOCKS: usize = 48;

#[tokio::test]
async fn large_upload_streams_beyond_old_ram_budget() {
    let server = Server::new().await;
    let user = server.user().await;
    let source = stream::iter(0..BLOCKS).map(|_| Ok::<Vec<u8>, std::io::Error>(vec![b'v'; BLOCK]));
    let response = server
        .call(Method::POST, "/store/stream.big")
        .bearer_auth(&user)
        .body(reqwest::Body::wrap_stream(source))
        .send()
        .await
        .unwrap();
    assert_eq!(
        response.status(),
        StatusCode::CREATED,
        "large upload must not be rejected by the 40 MiB buffer budget"
    );
    let size = (BLOCK * BLOCKS) as u64;
    assert_eq!(
        std::fs::metadata(server.home.join("store/stream.big"))
            .unwrap()
            .len(),
        size
    );
    let head = server
        .call(Method::HEAD, "/store/stream.big")
        .send()
        .await
        .unwrap();
    assert_eq!(head.status(), StatusCode::OK);
    assert!(head.headers().contains_key("etag"));
    let mut entries = std::fs::read_dir(server.home.join("store-tmp")).unwrap();
    assert!(
        entries.next().is_none(),
        "completed upload left staging files"
    );
}

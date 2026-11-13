use super::fixture::Server;
use reqwest::{Method, StatusCode};
use sha2::{Digest, Sha256};
mod concurrency;
#[tokio::test]
async fn concurrent_create_and_cas_have_one_winner() {
    let server = Server::new().await;
    let token = server.user().await;
    let path = "/store/contended";
    let statuses = concurrency::run(&server, Method::POST, path, &token, None).await;
    assert_eq!(
        statuses
            .iter()
            .filter(|&&s| s == StatusCode::CREATED)
            .count(),
        1
    );
    assert_eq!(
        statuses
            .iter()
            .filter(|&&s| s == StatusCode::CONFLICT)
            .count(),
        15
    );
    let data = server
        .call(Method::GET, path)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let digest = hex::encode(Sha256::digest(&data));
    let statuses = concurrency::run(&server, Method::PUT, path, &token, Some(&digest)).await;
    assert_eq!(statuses.iter().filter(|&&s| s == StatusCode::OK).count(), 1);
    assert_eq!(
        statuses
            .iter()
            .filter(|&&s| s == StatusCode::PRECONDITION_FAILED)
            .count(),
        15
    );
}

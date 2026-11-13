use super::Server;
use reqwest::{Method, StatusCode};
use std::fs;
mod worker;
#[tokio::test]
async fn concurrent_mixed_load_preserves_bytes_and_drains_journals() {
    let server = Server::new().await;
    let token = server.user().await;
    for index in 0..8 {
        let reply = server
            .call(Method::POST, &format!("/store/stress_{index}"))
            .bearer_auth(&token)
            .body("seed")
            .send()
            .await
            .unwrap();
        assert_eq!(reply.status(), StatusCode::CREATED);
    }
    let started = std::time::Instant::now();
    let mut tasks = Vec::new();
    for index in 0..8 {
        let client = server.client.clone();
        let endpoint = format!("{}/store/stress_{index}", server.address);
        let token = token.clone();
        tasks.push(tokio::spawn(worker::run(client, endpoint, token)));
    }
    for task in tasks {
        task.await.unwrap();
    }
    assert!(
        fs::read_dir(server.home.join("state/store-transactions"))
            .unwrap()
            .next()
            .is_none()
    );
    eprintln!(
        "verified 200 parallel read/CAS requests in {:?}",
        started.elapsed()
    );
}

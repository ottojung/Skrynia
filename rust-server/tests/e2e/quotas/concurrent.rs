use super::{
    Server,
    common::usage,
    grant_api::{create, issue},
    requests::send,
};
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn scope_creation_completes_under_concurrent_mutations() {
    let server = Server::new().await;
    let root = server.user().await;
    assert_eq!(
        send(&server, Method::POST, "team.key", &root, "seed").await,
        StatusCode::CREATED
    );
    let client = server.client.clone();
    let url = format!("{}/store/team.key", server.address);
    let writer = tokio::spawn({
        let token = root.clone();
        async move {
            for i in 0..40 {
                let reply = client
                    .put(&url)
                    .bearer_auth(&token)
                    .body(format!("version{i:08}"))
                    .send()
                    .await
                    .unwrap();
                assert_eq!(reply.status(), StatusCode::OK);
            }
        }
    });
    let parent = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        create(&server, &root, "team.*", 10000),
    )
    .await
    .unwrap();
    writer.await.unwrap();
    let credentials = issue(&server, &parent).await;
    let id = parent["id"].as_str().unwrap();
    let token = credentials["userToken"].as_str().unwrap();
    let totals = usage(&server, id, token).await;
    assert_eq!(totals["bytes"], 2048 + 512 + "version00000039".len());
    assert_eq!(totals["objects"], 1);
}

use super::Server;
use reqwest::{Method, StatusCode};
use std::fs;
#[tokio::test]
async fn failed_grant_persistence_before_rename_does_not_poison_server() {
    let server = Server::new().await;
    let (id, issuer) = server.root();
    let path = server.home.join("state/grants").join(format!("{id}.json"));
    let saved = path.with_extension("backup");
    fs::rename(&path, &saved).unwrap();
    fs::create_dir(&path).unwrap();
    let issue = || {
        server
            .call(Method::POST, &format!("/grants/{id}/users"))
            .bearer_auth(&issuer)
    };
    assert_eq!(
        issue().send().await.unwrap().status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    assert_eq!(
        server
            .call(Method::GET, "/health")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::OK
    );
    fs::remove_dir(path.clone()).unwrap();
    fs::rename(saved, path).unwrap();
    assert_eq!(issue().send().await.unwrap().status(), StatusCode::OK);
}

use super::Server;
use reqwest::{Method, StatusCode};
use serde_json::Value;
#[tokio::test]
async fn scoped_listing_is_paginated_and_requires_authority() {
    let server = Server::new().await;
    let token = server.user().await;
    for key in ["team.first", "team.second", "elsewhere"] {
        let reply = server
            .call(Method::POST, &format!("/store/{key}"))
            .bearer_auth(&token)
            .body("x")
            .send()
            .await
            .unwrap();
        assert_eq!(reply.status(), StatusCode::CREATED);
    }
    let reply = server
        .call(Method::GET, "/store?locations=team.*&limit=1")
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(reply.status(), StatusCode::OK);
    let page: Value = reply.json().await.unwrap();
    assert_eq!(page["keys"], serde_json::json!(["team.first"]));
    assert_eq!(page["next"], "team.first");
    let reply = server
        .call(
            Method::GET,
            "/store?locations=team.*&limit=1&after=team.first",
        )
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let page: Value = reply.json().await.unwrap();
    assert_eq!(page["keys"], serde_json::json!(["team.second"]));
    assert_eq!(
        server
            .call(Method::GET, "/store?locations=*")
            .send()
            .await
            .unwrap()
            .status(),
        StatusCode::UNAUTHORIZED
    );
}

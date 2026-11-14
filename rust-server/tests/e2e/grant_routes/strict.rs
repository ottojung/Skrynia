use super::Server;
use reqwest::{Method, StatusCode};
use serde_json::{Value, json};
#[tokio::test]
async fn malformed_delegation_never_inherits_parent_permissions() {
    let server = Server::new().await;
    let root = server.user().await;
    let base = json!({"locations":"team.*","operations":["create"],"maxBytes":4096,"maxObjects":5});
    let invalid = [
        json!({"location":"team.*","operations":["create"],"maxBytes":4096,"maxObjects":5}),
        json!({"locations":"team.*","operation":["create"],"maxBytes":4096,"maxObjects":5}),
        json!({"locations":null,"operations":["create"],"maxBytes":4096,"maxObjects":5}),
        json!({"locations":"team.*","operations":null,"maxBytes":4096,"maxObjects":5}),
        json!({"locations":"team.*","operations":["create"],"maxBytes":4096,"maxObjects":5,"typo":1}),
        json!({"maxBytes":4096,"maxObjects":5}),
    ];
    for spec in invalid {
        let result = server
            .call(Method::POST, "/grants")
            .bearer_auth(&root)
            .json(&spec)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::BAD_REQUEST, "{spec}");
    }
    let result = server
        .call(Method::POST, "/grants")
        .bearer_auth(&root)
        .json(&base)
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::OK);
    let accepted: Value = result.json().await.unwrap();
    assert_eq!(accepted["locations"], "team.*");
    assert_eq!(accepted["operations"], json!(["create"]));
}

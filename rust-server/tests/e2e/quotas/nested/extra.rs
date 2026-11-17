use super::{Method, Server, StatusCode, send};
pub async fn check(server: &Server, root: &str, child: &str) {
    assert_eq!(
        send(server, Method::POST, "apps/long", root, "no").await,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(server, Method::POST, "apps/long/x.css/deeper", root, "no").await,
        StatusCode::CONFLICT
    );
    let listed = server
        .call(Method::GET, "/store?locations=apps%2Flong%2F*&limit=10")
        .bearer_auth(child)
        .send()
        .await
        .unwrap();
    assert_eq!(listed.status(), StatusCode::OK);
    let listed: serde_json::Value = listed.json().await.unwrap();
    assert_eq!(listed["keys"].as_array().unwrap().len(), 2);
}

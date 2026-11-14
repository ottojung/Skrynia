use super::Server;
use reqwest::{Method, StatusCode};
pub async fn send(
    server: &Server,
    method: Method,
    key: &str,
    token: &str,
    data: &str,
) -> StatusCode {
    server
        .call(method, &format!("/store/{key}"))
        .bearer_auth(token)
        .body(data.to_owned())
        .send()
        .await
        .unwrap()
        .status()
}

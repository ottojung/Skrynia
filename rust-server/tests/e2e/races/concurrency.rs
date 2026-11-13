use super::Server;
use reqwest::{Method, StatusCode};
pub async fn run(
    server: &Server,
    method: Method,
    path: &str,
    token: &str,
    hash: Option<&str>,
) -> Vec<StatusCode> {
    let mut tasks = Vec::new();
    for index in 0..16 {
        let mut request = server
            .call(method.clone(), path)
            .bearer_auth(token)
            .body(format!(
                "{}-{index}",
                if method == Method::POST {
                    "value"
                } else {
                    "updated"
                }
            ));
        if let Some(hash) = hash {
            request = request.header("X-Skrynia-If-SHA256", hash);
        }
        tasks.push(tokio::spawn(async move {
            request.send().await.unwrap().status()
        }));
    }
    let mut outcomes = Vec::new();
    for task in tasks {
        outcomes.push(task.await.unwrap());
    }
    outcomes
}

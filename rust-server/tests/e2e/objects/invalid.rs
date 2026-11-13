use super::Server;
use reqwest::Method;
#[tokio::test]
async fn invalid_keys_and_unknown_routes() {
    let server = Server::new().await;
    for route in ["/store/.private", "/store/a%2Fb", "/unsupported/asset"] {
        let result = server.call(Method::GET, route).send().await.unwrap();
        assert!(
            !result.status().is_success(),
            "unexpected success for {route}"
        );
    }
}

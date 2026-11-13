use super::Server;
use reqwest::{Method, StatusCode};
#[tokio::test]
async fn unknown_range_units_and_multiple_ranges_fall_back_to_full_response() {
    let server = Server::new().await;
    let token = server.user().await;
    let path = "/store/ranged";
    let created = server
        .call(Method::POST, path)
        .bearer_auth(token)
        .body("0123456789")
        .send()
        .await
        .unwrap();
    assert_eq!(created.status(), StatusCode::CREATED);
    for header in ["items=0-1", "bytes=0-1,5-6", "bytes=nonsense"] {
        let result = server
            .call(Method::GET, path)
            .header("Range", header)
            .send()
            .await
            .unwrap();
        assert_eq!(result.status(), StatusCode::OK, "{header}");
        assert_eq!(result.text().await.unwrap(), "0123456789");
    }
    let result = server
        .call(Method::GET, path)
        .header("Range", "bytes=999-1000")
        .send()
        .await
        .unwrap();
    assert_eq!(result.status(), StatusCode::RANGE_NOT_SATISFIABLE);
}

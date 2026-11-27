mod list;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use list::{discovery_body, expected};
use tower::ServiceExt;

#[test]
fn version_exposes_typed_signatures_and_all_custom_names() {
    let discovery = super::provider::Discovery {
        status: 200,
        body: discovery_body(),
        ..Default::default()
    };
    let provider = super::provider::start(discovery, 2);
    let calls = libskrynia::Syscalls::http(&provider.url).unwrap();
    let app = libskrynia::http::router(super::kernel(super::ROOT, calls).unwrap());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let response = app
            .clone()
            .oneshot(Request::post("/version").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["version"].is_string() && json["commit"].is_string());
        assert_eq!(json["syscalls"], expected());
        let get = app
            .oneshot(Request::get("/version").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(get.status(), StatusCode::NOT_FOUND);
    });
    provider.join();
}

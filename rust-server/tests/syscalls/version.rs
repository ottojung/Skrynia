use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

#[tokio::test]
async fn discovery_lists_all_custom_names_without_executing_root() {
    let mut calls = libskrynia::Syscalls::new();
    calls
        .register("double", |_, _| panic!("version must not execute root"))
        .unwrap();
    calls.register("unused", |_, n| Ok(n)).unwrap();
    calls
        .http_fallback("http://127.0.0.1:1", &["remote", "double"])
        .unwrap();
    let app = libskrynia::http::router(super::kernel(super::ROOT, calls).unwrap());
    let response = app
        .clone()
        .oneshot(Request::post("/version").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["version"].is_string() && json["commit"].is_string());
    assert_eq!(
        json["syscalls"],
        serde_json::json!(["double", "remote", "unused"])
    );
    let get = app
        .oneshot(Request::get("/version").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(get.status(), StatusCode::NOT_FOUND);
}

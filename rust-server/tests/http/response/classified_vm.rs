use super::kernel_with_wasm;
use axum::Router;
use axum::body::Body;
use axum::http::header::RETRY_AFTER;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use skrynia_server::http;
use tower::ServiceExt;

const LIMIT_WAT: &str = r#"(module (memory (export "memory") 1) (func (export "skrynia_main") (param i32 i32) (result i32) (i32.const -10)))"#;
const FAULT_WAT: &str = r#"(module (memory (export "memory") 1) (func (export "skrynia_main") (param i32 i32) (result i32) (i32.const -9)))"#;

fn app(name: &str, wat: &str) -> Router {
    http::router(kernel_with_wasm(name, &wat::parse_str(wat).unwrap()))
}

async fn post(app: Router, body: &'static str) -> Response {
    app.oneshot(Request::post("/").body(Body::from(body)).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn vm_limit_exit_is_classified_nonempty_503() {
    let res = post(app("v3-cfg-limit", LIMIT_WAT), "").await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.headers().get(RETRY_AFTER).unwrap(), "1");
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&body);
    assert!(!text.is_empty());
    assert!(text.contains("resource limit"));
}

#[tokio::test]
async fn vm_fault_exit_is_classified_nonempty_500() {
    let res = post(app("v3-cfg-fault", FAULT_WAT), "").await;
    assert_eq!(res.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&body);
    assert!(!text.is_empty());
    assert!(text.contains("guest fault"));
}

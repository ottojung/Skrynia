use super::kernel_with_wasm;
use axum::Router;
use axum::body::Body;
use axum::http::header::RETRY_AFTER;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use skrynia_server::http;
use tower::ServiceExt;

const STALL: &str = r#"(module (import "skrynia" "write" (func $w (param i32 i32 i32) (result i32))) (memory (export "memory") 2) (data (i32.const 0) "L") (func (export "skrynia_main") (param i32 i32) (result i32) (local $i i32) (memory.fill (i32.const 0) (i32.const 76) (i32.const 16384)) (loop $a (drop (call $w (local.get 1) (i32.const 0) (i32.const 16384))) (local.set $i (i32.add (local.get $i) (i32.const 1))) (br_if $a (i32.lt_u (local.get $i) (i32.const 832)))) (i32.const 0)))"#;

fn app() -> Router {
    http::router(kernel_with_wasm(
        "v3-cfg-permit",
        &wat::parse_str(STALL).unwrap(),
    ))
}

async fn post(app: Router, body: &'static str) -> Response {
    app.oneshot(Request::post("/").body(Body::from(body)).unwrap())
        .await
        .unwrap()
}

#[tokio::test]
async fn permit_exhaustion_is_classified_nonempty_503() {
    let app = app();
    let mut held = Vec::new();
    for _ in 0..16 {
        let r = post(app.clone(), "x").await;
        assert_eq!(r.status(), StatusCode::OK);
        held.push(r);
    }
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let res = post(app, "").await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(res.headers().get(RETRY_AFTER).unwrap(), "1");
    let body = res.into_body().collect().await.unwrap().to_bytes();
    let text = String::from_utf8_lossy(&body);
    assert!(!text.is_empty());
    assert!(text.contains("resource limit"));
}

use super::basic::kernel_with_wasm;
use axum::{
    body::{Body, Bytes},
    http::{Request, StatusCode},
};
use futures_util::stream;
use skrynia_server::http;
use tower::ServiceExt;
#[tokio::test]
async fn root_can_return_without_consuming_endless_input() {
    let code = wat::parse_str(
        r#"(module (memory(export "memory") 1)
        (func(export "skrynia_main")(param i32 i32 i32)(result i32)
          (i32.store8 (local.get 1) (i32.const 88)) (i32.const 1)))"#,
    )
    .unwrap();
    let app = http::router(kernel_with_wasm("skrynia-root-early", &code));
    let stream = stream::pending::<Result<Bytes, std::io::Error>>();
    let req = Request::post("/").body(Body::from_stream(stream)).unwrap();
    let out = tokio::time::timeout(std::time::Duration::from_secs(2), app.oneshot(req))
        .await
        .expect("root must cancel unread HTTP body")
        .unwrap();
    assert_eq!(out.status(), StatusCode::OK);
}

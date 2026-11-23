use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use skrynia_server::http;
use tower::ServiceExt;
#[tokio::test]
async fn concurrent_requests_with_nonzero_guest() {
    let raw = [0x61u8; 32];
    let (k, dir) = super::setup("conc", &super::echo_guest(), &raw);
    let app = http::router(k);
    let mut joins = Vec::new();
    for i in 0u8..8 {
        let app = app.clone();
        let tok = raw;
        let payload = vec![b'a' + i; 4096 + i as usize];
        joins.push(tokio::spawn(async move {
            let req = Request::post("/")
                .body(Body::from(super::body(&tok, &payload)))
                .unwrap();
            let res = app.oneshot(req).await.unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let got = res.into_body().collect().await.unwrap().to_bytes();
            assert_eq!(got.as_ref(), payload.as_slice());
        }));
    }
    for j in joins {
        j.await.unwrap();
    }
    let _ = std::fs::remove_dir_all(dir);
}

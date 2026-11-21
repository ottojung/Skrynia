use axum::{body::Body, http::Request, http::StatusCode};
use http_body_util::BodyExt;
use skrynia_server::{boot, http};
use std::fs;
use tower::ServiceExt;

fn kernel(name: &str) -> (boot::Kernel, std::path::PathBuf) {
    let root = include_str!("../../../../test-fixtures/mutation_lock_root.wat");
    let dir = std::env::temp_dir().join(format!("http-mut-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("operator")).unwrap();
    fs::create_dir_all(dir.join("user")).unwrap();
    fs::write(dir.join("root.wasm"), wat::parse_str(root).unwrap()).unwrap();
    (
        boot::Kernel::start_with_roots(
            &dir.join("root.wasm"),
            &dir.join("operator"),
            &dir.join("user"),
        )
        .unwrap(),
        dir,
    )
}

#[tokio::test]
async fn parallel_requests_serialize_mutation_lock() {
    let (k, dir) = kernel("par");
    k.ram().truncate(16).unwrap();
    let app = http::router(k);
    let mut joins = Vec::new();
    for _ in 0..8 {
        let app = app.clone();
        joins.push(tokio::spawn(async move {
            for _ in 0..20 {
                let req = Request::post("/").body(Body::from("x")).unwrap();
                let res = app.clone().oneshot(req).await.unwrap();
                assert_eq!(res.status(), StatusCode::OK);
                let b = res.into_body().collect().await.unwrap().to_bytes();
                assert!(b.as_ref() == b"0" || b.as_ref() == b"1");
            }
        }));
    }
    for j in joins {
        j.await.unwrap();
    }
    let _ = fs::remove_dir_all(dir);
}

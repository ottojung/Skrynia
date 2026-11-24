use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use libskrynia::{boot, conf, http};
use std::{ffi::OsString, fs};
use tower::ServiceExt;
#[rustfmt::skip]
fn root() -> Vec<u8> {
    wat::parse_str(concat!(
        "(module(import \"skrynia\" \"open_disk\"(func $o(param i32 i32 i32)(result i32)))",
        "(import \"skrynia\" \"read\"(func $r(param i32 i32 i32)(result i32)))",
        "(import \"skrynia\" \"close\"(func $c(param i32)(result i32)))",
        "(import \"skrynia\" \"write\"(func $w(param i32 i32 i32)(result i32)))",
        "(memory(export \"memory\") 2)(data(i32.const 0)\"secret.bin\")",
        "(func(export \"skrynia_main\")(param $in i32)(param $out i32)(result i32)",
        "(local $fd i32)(local $n i32)",
        "(local.set $fd(call $o(i32.const 0)(i32.const 10)(i32.const 1)))",
        "(if(i32.lt_s(local.get $fd)(i32.const 0))(then(return(i32.const 0))))",
        "(local.set $n(call $r(local.get $fd)(i32.const 32)(i32.const 64)))",
        "(drop(call $c(local.get $fd)))",
        "(if(i32.gt_s(local.get $n)(i32.const 0))(then",
        "(drop(call $w(local.get $out)(i32.const 32)(local.get $n)))))",
        "(i32.const 0)))")).unwrap()
}
#[rustfmt::skip]
#[tokio::test]
async fn no_user_shadow_for_missing_operator_file() {
    let d = std::env::temp_dir().join(format!("sk-pr119-shadow-{}-{}", std::process::id(), rand::random::<u64>()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(d.join("op")).unwrap();
    fs::create_dir_all(d.join("user")).unwrap();
    fs::write(d.join("user/secret.bin"), b"SHADOW").unwrap();
    fs::write(d.join("root.wasm"), root()).unwrap();
    let cfg = conf::Config::from_lookup(|k| match k {
        "SKRYNIA_ROOT_WASM" => Some(OsString::from(d.join("root.wasm"))),
        "SKRYNIA_OPERATOR_DIR" => Some(OsString::from(d.join("op"))),
        "SKRYNIA_USER_DIR" => Some(OsString::from(d.join("user"))),
        _ => None,
    }).unwrap();
    let app = http::router(boot::Kernel::start(&cfg.root_wasm, &cfg.operator_dir, &cfg.user_dir).unwrap());
    let r = app.oneshot(Request::post("/").body(Body::from("x")).unwrap()).await.unwrap();
    assert_eq!(r.status(), StatusCode::OK);
    let b = r.into_body().collect().await.unwrap().to_bytes();
    assert_ne!(b.as_ref(), b"SHADOW");
    let _ = fs::remove_dir_all(d);
}

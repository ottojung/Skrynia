use super::replies;
use axum::http::Method;
use axum::response::{IntoResponse, Response};

pub fn mounted_path(base: &str, uri: &str) -> String {
    if base.is_empty() {
        return uri.into();
    }
    if uri == base {
        return "/".into();
    }
    uri.strip_prefix(&format!("{base}/"))
        .map(|x| format!("/{x}"))
        .unwrap_or_default()
}
pub fn public(path: &str, method: &Method) -> Option<Response> {
    if method != Method::GET {
        return None;
    }
    match path {
        "/health" => Some(replies::ok(serde_json::json!({"ok":true}))),
        "/version" => Some(replies::ok(
            serde_json::json!({"version":"development","commit":"development"}),
        )),
        "/client/skrynia.js" => Some(
            (
                [
                    ("content-type", "text/javascript"),
                    ("cache-control", "no-cache"),
                ],
                include_str!("../../../../src/client.js"),
            )
                .into_response(),
        ),
        _ => None,
    }
}

pub fn failed(state: &crate::State) -> Option<Response> {
    if !state.failed.load(std::sync::atomic::Ordering::Acquire) {
        return None;
    }
    Some(
        crate::Fail(
            axum::http::StatusCode::SERVICE_UNAVAILABLE,
            "recovery_required",
        )
        .into_response(),
    )
}

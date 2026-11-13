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
        "/version" => Some(replies::ok(super::build::info())),
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

#[cfg(test)]
mod tests {
    use super::public;
    #[test]
    fn only_server_health_and_version_are_public_utilities() {
        let method = axum::http::Method::GET;
        assert!(public("/health", &method).is_some());
        assert!(public("/version", &method).is_some());
    }
}

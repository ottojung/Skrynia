use axum::http::{HeaderMap, StatusCode};
pub fn token(headers: &HeaderMap) -> crate::error::Result<&str> {
    let text = headers
        .get("authorization")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    let Some(value) = text.strip_prefix("Bearer ") else {
        return Err(crate::error::Fail(
            StatusCode::UNAUTHORIZED,
            "grant_user_token_required",
        ));
    };
    if !crate::valid_sha256(value) {
        return Err(crate::error::Fail(
            StatusCode::UNAUTHORIZED,
            "grant_user_token_required",
        ));
    }
    Ok(value)
}

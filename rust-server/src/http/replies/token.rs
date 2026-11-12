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
    if value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    {
        return Err(crate::error::Fail(
            StatusCode::UNAUTHORIZED,
            "grant_user_token_required",
        ));
    }
    Ok(value)
}

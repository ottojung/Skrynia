use crate::{Result, bad};
use axum::http::{HeaderMap, Method};

pub fn precondition(headers: &HeaderMap, method: &Method) -> Result<Option<String>> {
    for header in [
        "if-match",
        "if-none-match",
        "if-modified-since",
        "if-unmodified-since",
        "if-range",
    ] {
        if headers.contains_key(header) {
            return Err(bad("invalid_precondition"));
        }
    }
    let Some(value) = headers.get("x-skrynia-if-sha256") else {
        return Ok(None);
    };
    let value = value.to_str().map_err(|_| bad("invalid_precondition"))?;
    if method == Method::POST
        || value.len() != 64
        || !value
            .bytes()
            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
    {
        return Err(bad("invalid_precondition"));
    }
    Ok(Some(value.into()))
}
pub fn key(value: &str) -> Result<String> {
    let key = form_urlencoded::parse(format!("x={value}").as_bytes())
        .find(|(k, _)| k == "x")
        .map(|(_, v)| v.to_string())
        .unwrap_or_default();
    if !crate::valid_key(&key) {
        return Err(bad("invalid_key"));
    }
    Ok(key)
}

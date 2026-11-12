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
    let raw = value.as_bytes();
    for (index, byte) in raw.iter().copied().enumerate() {
        if byte == b'%'
            && (index + 2 >= raw.len()
                || !raw[index + 1].is_ascii_hexdigit()
                || !raw[index + 2].is_ascii_hexdigit())
        {
            return Err(bad("invalid_percent_encoding"));
        }
    }
    let decoded = percent_encoding::percent_decode_str(value)
        .decode_utf8()
        .map_err(|_| bad("invalid_percent_encoding"))?;
    if !crate::valid_key(&decoded) {
        return Err(bad("invalid_key"));
    }
    Ok(decoded.into_owned())
}
#[cfg(test)]
mod tests;

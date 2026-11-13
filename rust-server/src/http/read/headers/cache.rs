use axum::http::HeaderMap;
pub fn fresh(headers: &HeaderMap, tag: &str) -> bool {
    headers
        .get("if-none-match")
        .and_then(|x| x.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .map(str::trim)
                .any(|x| x == "*" || x == tag || x == format!("W/{tag}"))
        })
}

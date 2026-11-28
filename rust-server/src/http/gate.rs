use axum::http::Request;
pub type Verdict = std::sync::Arc<std::sync::atomic::AtomicU8>;

pub fn declared_too_large<T>(r: &Request<T>, max: usize) -> bool {
    r.headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<usize>().ok())
        .is_some_and(|n| n > max)
}

pub fn chunked<T>(r: &Request<T>) -> bool {
    r.headers()
        .get("transfer-encoding")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v.to_ascii_lowercase().contains("chunked"))
}

use axum::http::StatusCode;

pub fn parse(header: &str, size: u64) -> Result<(u64, u64), StatusCode> {
    let value = header
        .strip_prefix("bytes=")
        .ok_or(StatusCode::RANGE_NOT_SATISFIABLE)?;
    let (a, b) = value
        .split_once('-')
        .ok_or(StatusCode::RANGE_NOT_SATISFIABLE)?;
    if size == 0 || a.is_empty() && b.is_empty() || b.contains('-') {
        return Err(StatusCode::RANGE_NOT_SATISFIABLE);
    }
    if a.is_empty() {
        let suffix: u64 = b.parse().map_err(|_| StatusCode::RANGE_NOT_SATISFIABLE)?;
        if suffix == 0 {
            return Err(StatusCode::RANGE_NOT_SATISFIABLE);
        }
        return Ok((size.saturating_sub(suffix), size - 1));
    }
    let start: u64 = a.parse().map_err(|_| StatusCode::RANGE_NOT_SATISFIABLE)?;
    let end: u64 = if b.is_empty() {
        size - 1
    } else {
        b.parse().map_err(|_| StatusCode::RANGE_NOT_SATISFIABLE)?
    };
    if start >= size || end < start {
        return Err(StatusCode::RANGE_NOT_SATISFIABLE);
    }
    Ok((start, end.min(size - 1)))
}

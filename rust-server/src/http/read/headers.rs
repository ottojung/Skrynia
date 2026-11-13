use super::range;
use axum::http::{HeaderMap, StatusCode};

pub struct Selection {
    pub status: StatusCode,
    pub start: u64,
    pub end: u64,
}
pub fn select(h: &HeaderMap, tag: &str, size: u64) -> Selection {
    let fresh = h
        .get("if-none-match")
        .and_then(|x| x.to_str().ok())
        .is_some_and(|v| {
            v.split(',')
                .map(str::trim)
                .any(|x| x == "*" || x == tag || x == format!("W/{tag}"))
        });
    if fresh {
        return Selection {
            status: StatusCode::NOT_MODIFIED,
            start: 0,
            end: 0,
        };
    }
    if let Some(header) = h.get("range")
        && h.get("if-range")
            .is_none_or(|v| v.as_bytes() == tag.as_bytes())
    {
        return match range::parse(header.to_str().unwrap_or(""), size) {
            Ok((start, end)) => Selection {
                status: StatusCode::PARTIAL_CONTENT,
                start,
                end,
            },
            Err(status) => Selection {
                status,
                start: 0,
                end: 0,
            },
        };
    }
    Selection {
        status: StatusCode::OK,
        start: 0,
        end: size.saturating_sub(1),
    }
}

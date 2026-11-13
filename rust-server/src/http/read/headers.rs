mod cache;
use super::range;
use axum::http::{HeaderMap, StatusCode};

pub struct Selection {
    pub status: StatusCode,
    pub start: u64,
    pub end: u64,
}
pub fn select(h: &HeaderMap, tag: &str, size: u64) -> Selection {
    if cache::fresh(h, tag) {
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
            Err(error) => {
                let ignored = matches!(error, range::Error::Ignore);
                Selection {
                    status: if ignored {
                        StatusCode::OK
                    } else {
                        StatusCode::RANGE_NOT_SATISFIABLE
                    },
                    start: 0,
                    end: if ignored { size.saturating_sub(1) } else { 0 },
                }
            }
        };
    }
    Selection {
        status: StatusCode::OK,
        start: 0,
        end: size.saturating_sub(1),
    }
}

use super::headers::Selection;
use crate::{Result, internal};
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::response::Response;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use tokio_util::io::ReaderStream;

pub fn respond(
    mut file: File,
    key: &str,
    tag: &str,
    size: u64,
    s: Selection,
    method: &Method,
) -> Result<Response> {
    let mut response = Response::new(Body::empty());
    *response.status_mut() = s.status;
    super::metadata::set(&mut response, key, tag);
    let headers = response.headers_mut();
    if s.status == StatusCode::RANGE_NOT_SATISFIABLE {
        headers.insert(
            "content-range",
            HeaderValue::from_str(&format!("bytes */{size}")).unwrap(),
        );
    } else if s.status != StatusCode::NOT_MODIFIED {
        if s.status == StatusCode::PARTIAL_CONTENT {
            headers.insert(
                "content-range",
                HeaderValue::from_str(&format!("bytes {}-{}/{size}", s.start, s.end)).unwrap(),
            );
        }
        let length = if size == 0 { 0 } else { s.end - s.start + 1 };
        headers.insert(
            "content-length",
            HeaderValue::from_str(&length.to_string()).unwrap(),
        );
        if method != Method::HEAD && length > 0 {
            file.seek(SeekFrom::Start(s.start))
                .map_err(|_| internal())?;
            let stream = tokio::io::AsyncReadExt::take(tokio::fs::File::from_std(file), length);
            *response.body_mut() =
                Body::from_stream(ReaderStream::with_capacity(stream, 64 * 1024));
        }
    }
    Ok(response)
}

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug)]
pub struct Fail(pub StatusCode, pub &'static str);
impl IntoResponse for Fail {
    fn into_response(self) -> Response {
        (self.0, axum::Json(serde_json::json!({"error": self.1}))).into_response()
    }
}
pub type Result<T> = std::result::Result<T, Fail>;
pub fn bad(code: &'static str) -> Fail {
    Fail(StatusCode::BAD_REQUEST, code)
}
pub fn forbidden(code: &'static str) -> Fail {
    Fail(StatusCode::FORBIDDEN, code)
}
pub fn missing(code: &'static str) -> Fail {
    Fail(StatusCode::NOT_FOUND, code)
}
pub fn internal() -> Fail {
    Fail(StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
}

pub fn transient_io(error: &std::io::Error) -> Fail {
    let full = matches!(
        crate::store::disk::write_errno(error),
        Some(libc::ENOSPC | libc::EDQUOT)
    );
    if full {
        Fail(StatusCode::INSUFFICIENT_STORAGE, "insufficient_storage")
    } else {
        Fail(StatusCode::SERVICE_UNAVAILABLE, "storage_unavailable")
    }
}
#[cfg(test)]
mod tests {
    use super::transient_io;
    #[test]
    fn exhausted_space_is_507_and_other_transient_io_is_503() {
        let storage = std::io::Error::from_raw_os_error(libc::ENOSPC);
        let quota = std::io::Error::from_raw_os_error(libc::EDQUOT);
        assert_eq!(transient_io(&storage).0.as_u16(), 507);
        assert_eq!(transient_io(&quota).0.as_u16(), 507);
        assert_eq!(transient_io(&std::io::Error::other("io")).0.as_u16(), 503);
    }
}

use super::{pump, term};
use axum::http::StatusCode;
use std::io;

#[derive(Debug, PartialEq)]
pub(super) enum Reject {
    TooLarge,
    BadRequest,
}

impl Reject {
    pub(super) fn status(self) -> StatusCode {
        match self {
            Self::TooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::BadRequest => StatusCode::BAD_REQUEST,
        }
    }

    pub(super) fn recorded(self, t: &term::Term) -> StatusCode {
        let e = match self {
            Self::TooLarge => io::Error::from(io::ErrorKind::InvalidData),
            Self::BadRequest => pump::stalled(),
        };
        term::set(t, &e);
        self.status()
    }
}

use super::wenc::encode_res;
use super::wire::{Response, STATUS_ERR, STATUS_OK};

pub(super) fn ok(body: Vec<u8>) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_OK,
        body,
    })
}
pub(super) fn err(code: u8) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_ERR,
        body: vec![code],
    })
}

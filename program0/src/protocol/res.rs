use crate::wenc::encode_res;
use crate::wire::{Response, STATUS_ERR, STATUS_OK};
pub fn ok(body: Vec<u8>) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_OK,
        body,
    })
}
pub fn err(code: u8) -> Vec<u8> {
    encode_res(&Response {
        status: STATUS_ERR,
        body: vec![code],
    })
}

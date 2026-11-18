use super::take::{DecodeErr, blob, take};
use crate::wire::{Response, VERSION};
pub fn decode_res(mut src: &[u8]) -> Result<Response, DecodeErr> {
    if take(&mut src, 1)?[0] != VERSION {
        return Err(DecodeErr::Version);
    }
    let status = take(&mut src, 1)?[0];
    let body = blob(&mut src)?;
    if !src.is_empty() {
        return Err(DecodeErr::Short);
    }
    Ok(Response { status, body })
}

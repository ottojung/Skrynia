use super::wdec::{DecodeErr, take};
use super::wire::{Response, VERSION};

pub fn decode_res(mut src: &[u8]) -> Result<Response, DecodeErr> {
    if take(&mut src, 1)?[0] != VERSION {
        return Err(DecodeErr::Version);
    }
    let status = take(&mut src, 1)?[0];
    let n = u32::from_le_bytes(take(&mut src, 4)?.try_into().unwrap()) as usize;
    let body = take(&mut src, n)?.to_vec();
    if !src.is_empty() {
        return Err(DecodeErr::Short);
    }
    Ok(Response { status, body })
}

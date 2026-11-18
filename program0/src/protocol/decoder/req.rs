use super::{
    ops,
    take::{DecodeErr, take},
};
use crate::wire::{Request, VERSION};
pub fn decode_req(mut src: &[u8]) -> Result<Request, DecodeErr> {
    if take(&mut src, 1)?[0] != VERSION {
        return Err(DecodeErr::Version);
    }
    let op = take(&mut src, 1)?[0];
    let out = ops::parse(op, &mut src)?;
    if !src.is_empty() {
        return Err(DecodeErr::Short);
    }
    Ok(out)
}

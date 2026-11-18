use super::take::{DecodeErr, blob, id};
use crate::wire::{OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET, Request};
pub fn parse(op: u8, src: &mut &[u8]) -> Result<Request, DecodeErr> {
    match op {
        OP_REGISTER => Ok(Request::Register { code: blob(src)? }),
        OP_SET => Ok(Request::Set {
            id: id(src)?,
            code: blob(src)?,
        }),
        OP_INVOKE => Ok(Request::Invoke {
            id: id(src)?,
            input: blob(src)?,
        }),
        OP_LOOKUP => Ok(Request::Lookup { id: id(src)? }),
        OP_REVOKE => Ok(Request::Revoke { id: id(src)? }),
        _ => Err(DecodeErr::Op),
    }
}

pub use super::wdec_res::decode_res;
use super::wire::{OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET, Request, VERSION};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeErr {
    Short,
    Version,
    Op,
}

pub(super) fn take<'a>(src: &mut &'a [u8], n: usize) -> Result<&'a [u8], DecodeErr> {
    if src.len() < n {
        return Err(DecodeErr::Short);
    }
    let (h, t) = src.split_at(n);
    *src = t;
    Ok(h)
}

fn id(src: &mut &[u8]) -> Result<Vec<u8>, DecodeErr> {
    let n = u16::from_le_bytes(take(src, 2)?.try_into().unwrap()) as usize;
    Ok(take(src, n)?.to_vec())
}

fn blob(src: &mut &[u8]) -> Result<Vec<u8>, DecodeErr> {
    let n = u32::from_le_bytes(take(src, 4)?.try_into().unwrap()) as usize;
    Ok(take(src, n)?.to_vec())
}

pub fn decode_req(mut src: &[u8]) -> Result<Request, DecodeErr> {
    if take(&mut src, 1)?[0] != VERSION {
        return Err(DecodeErr::Version);
    }
    let req = match take(&mut src, 1)?[0] {
        OP_REGISTER => Request::Register {
            code: blob(&mut src)?,
        },
        OP_SET => Request::Set {
            id: id(&mut src)?,
            code: blob(&mut src)?,
        },
        OP_INVOKE => Request::Invoke {
            id: id(&mut src)?,
            input: blob(&mut src)?,
        },
        OP_LOOKUP => Request::Lookup { id: id(&mut src)? },
        OP_REVOKE => Request::Revoke { id: id(&mut src)? },
        _ => return Err(DecodeErr::Op),
    };
    if !src.is_empty() {
        return Err(DecodeErr::Short);
    }
    Ok(req)
}

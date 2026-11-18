use crate::wire::{
    OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET, Request, Response, VERSION,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeErr {
    Short,
    Version,
    Op,
}

fn take<'a>(src: &mut &'a [u8], n: usize) -> Result<&'a [u8], DecodeErr> {
    if src.len() < n {
        return Err(DecodeErr::Short);
    }
    let (h, t) = src.split_at(n);
    *src = t;
    Ok(h)
}

pub fn decode_req(mut src: &[u8]) -> Result<Request, DecodeErr> {
    if take(&mut src, 1)?[0] != VERSION {
        return Err(DecodeErr::Version);
    }
    match take(&mut src, 1)?[0] {
        OP_REGISTER => {
            let n = u32::from_le_bytes(take(&mut src, 4)?.try_into().unwrap()) as usize;
            let code = take(&mut src, n)?.to_vec();
            if !src.is_empty() {
                return Err(DecodeErr::Short);
            }
            Ok(Request::Register { code })
        }
        OP_SET => {
            let il = u16::from_le_bytes(take(&mut src, 2)?.try_into().unwrap()) as usize;
            let id = take(&mut src, il)?.to_vec();
            let n = u32::from_le_bytes(take(&mut src, 4)?.try_into().unwrap()) as usize;
            let code = take(&mut src, n)?.to_vec();
            if !src.is_empty() {
                return Err(DecodeErr::Short);
            }
            Ok(Request::Set { id, code })
        }
        OP_INVOKE => {
            let il = u16::from_le_bytes(take(&mut src, 2)?.try_into().unwrap()) as usize;
            let id = take(&mut src, il)?.to_vec();
            let n = u32::from_le_bytes(take(&mut src, 4)?.try_into().unwrap()) as usize;
            let input = take(&mut src, n)?.to_vec();
            if !src.is_empty() {
                return Err(DecodeErr::Short);
            }
            Ok(Request::Invoke { id, input })
        }
        OP_LOOKUP => {
            let il = u16::from_le_bytes(take(&mut src, 2)?.try_into().unwrap()) as usize;
            let id = take(&mut src, il)?.to_vec();
            if !src.is_empty() {
                return Err(DecodeErr::Short);
            }
            Ok(Request::Lookup { id })
        }
        OP_REVOKE => {
            let il = u16::from_le_bytes(take(&mut src, 2)?.try_into().unwrap()) as usize;
            let id = take(&mut src, il)?.to_vec();
            if !src.is_empty() {
                return Err(DecodeErr::Short);
            }
            Ok(Request::Revoke { id })
        }
        _ => Err(DecodeErr::Op),
    }
}

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

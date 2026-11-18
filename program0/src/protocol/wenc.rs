use crate::wire::{
    OP_INVOKE, OP_LOOKUP, OP_REGISTER, OP_REVOKE, OP_SET, Request, Response, VERSION,
};

pub fn encode_req(req: &Request) -> Vec<u8> {
    let mut out = vec![VERSION];
    match req {
        Request::Register { code } => {
            out.push(OP_REGISTER);
            out.extend_from_slice(&(code.len() as u32).to_le_bytes());
            out.extend_from_slice(code);
        }
        Request::Set { id, code } => {
            out.push(OP_SET);
            out.extend_from_slice(&(id.len() as u16).to_le_bytes());
            out.extend_from_slice(id);
            out.extend_from_slice(&(code.len() as u32).to_le_bytes());
            out.extend_from_slice(code);
        }
        Request::Invoke { id, input } => {
            out.push(OP_INVOKE);
            out.extend_from_slice(&(id.len() as u16).to_le_bytes());
            out.extend_from_slice(id);
            out.extend_from_slice(&(input.len() as u32).to_le_bytes());
            out.extend_from_slice(input);
        }
        Request::Lookup { id } => {
            out.push(OP_LOOKUP);
            out.extend_from_slice(&(id.len() as u16).to_le_bytes());
            out.extend_from_slice(id);
        }
        Request::Revoke { id } => {
            out.push(OP_REVOKE);
            out.extend_from_slice(&(id.len() as u16).to_le_bytes());
            out.extend_from_slice(id);
        }
    }
    out
}

pub fn encode_res(res: &Response) -> Vec<u8> {
    let mut out = vec![VERSION, res.status];
    out.extend_from_slice(&(res.body.len() as u32).to_le_bytes());
    out.extend_from_slice(&res.body);
    out
}

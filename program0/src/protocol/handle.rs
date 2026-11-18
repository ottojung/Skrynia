use super::cap::{Disk, Eval, Random};
use super::res::{err, ok};
use crate::reg::Registry;
use crate::wdec::decode_req;
use crate::wire::Request;

pub fn handle<D: Disk, R: Random, E: Eval>(
    reg: &Registry,
    disk: &D,
    rng: &R,
    eval: &E,
    req: &[u8],
) -> Vec<u8> {
    let req = match decode_req(req) {
        Ok(r) => r,
        Err(_) => return err(1),
    };
    match req {
        Request::Register { code } => match reg.register(disk, rng, code) {
            Ok(id) => ok(id),
            Err(_) => err(2),
        },
        Request::Set { id, code } => match reg.set(disk, &id, code) {
            Ok(()) => ok(Vec::new()),
            Err(_) => err(3),
        },
        Request::Invoke { id, input } => match reg.invoke(disk, eval, &id, &input) {
            Ok(out) => ok(out),
            Err(_) => err(4),
        },
        Request::Lookup { id } => match reg.lookup(&id) {
            Ok(code) => ok(code),
            Err(_) => err(5),
        },
        Request::Revoke { id } => match reg.revoke(disk, &id) {
            Ok(()) => ok(Vec::new()),
            Err(_) => err(6),
        },
    }
}

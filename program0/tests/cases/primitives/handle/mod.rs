use crate::cases::support::{MemDisk, Rng};
use skrynia_program0::cap::Eval;
use skrynia_program0::handle::handle;
use skrynia_program0::reg::Registry;
use skrynia_program0::wdec::decode_res;
use skrynia_program0::wenc::encode_req;
use skrynia_program0::wire::{Request, STATUS_ERR, STATUS_OK};

struct CatEval;
impl Eval for CatEval {
    type Err = ();
    fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
        Ok([w, b":", i].concat())
    }
}

mod case0;
mod case1;
mod case2;

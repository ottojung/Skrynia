use super::*;
pub(super) struct SelfEval {
    pub(super) reg: Arc<Registry>,
    pub(super) disk: Arc<MemDisk>,
    pub(super) me: Vec<u8>,
}
impl Eval for SelfEval {
    type Err = ();
    fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
        if w == b"self" && !i.is_empty() {
            self.reg
                .invoke(&*self.disk, self, &self.me, &i[1..])
                .map_err(|_| ())
        } else {
            Ok(vec![i.len() as u8])
        }
    }
}

pub(super) struct CycleEval {
    pub(super) reg: Arc<Registry>,
    pub(super) disk: Arc<MemDisk>,
    pub(super) a: Vec<u8>,
    pub(super) b: Vec<u8>,
}
impl Eval for CycleEval {
    type Err = ();
    fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
        match w {
            b"A" => self
                .reg
                .invoke(&*self.disk, self, &self.b, i)
                .map_err(|_| ()),
            b"B" if !i.is_empty() => self
                .reg
                .invoke(&*self.disk, self, &self.a, &i[1..])
                .map_err(|_| ()),
            _ => Ok(b"done".to_vec()),
        }
    }
}

use super::reg::{RegErr, Registry};

impl Registry {
    pub fn invoke<D: crate::cap::Disk, E: crate::cap::Eval>(
        &self,
        _disk: &D,
        eval: &E,
        pid: &[u8],
        input: &[u8],
    ) -> Result<Vec<u8>, RegErr<D::Err, E::Err, ()>> {
        let code = {
            self.code
                .read()
                .unwrap()
                .get(pid)
                .cloned()
                .ok_or(RegErr::Missing)?
        };
        eval.eval(&code, input).map_err(RegErr::Eval)
    }
}

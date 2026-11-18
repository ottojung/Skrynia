use crate::cap::{Disk, Eval, Random};
use crate::id::ID_LEN;
use crate::reg::{RegErr, Registry};
use crate::store_io;

impl Registry {
    pub fn register<D: Disk, R: Random>(&self, disk: &D, rng: &R, code: Vec<u8>)
        -> Result<Vec<u8>, RegErr<D::Err, (), R::Err>> {
        let mut pid = vec![0u8; ID_LEN];
        rng.fill(&mut pid).map_err(RegErr::Rand)?;
        store_io::put(disk, &self.dir, &pid, &code).map_err(RegErr::Store)?;
        self.code.write().unwrap().insert(pid.clone(), code);
        Ok(pid)
    }
    pub fn set<D: Disk>(&self, disk: &D, pid: &[u8], code: Vec<u8>)
        -> Result<(), RegErr<D::Err, (), ()>> {
        let mut map = self.code.write().unwrap();
        if map.get(pid).is_none() { return Err(RegErr::Missing); }
        store_io::put(disk, &self.dir, pid, &code).map_err(RegErr::Store)?;
        map.insert(pid.to_vec(), code);
        Ok(())
    }
    pub fn invoke<D: Disk, E: Eval>(&self, _disk: &D, eval: &E, pid: &[u8], input: &[u8])
        -> Result<Vec<u8>, RegErr<D::Err, E::Err, ()>> {
        let code = { self.code.read().unwrap().get(pid).cloned().ok_or(RegErr::Missing)? };
        eval.eval(&code, input).map_err(RegErr::Eval)
    }
}

use super::id::ID_LEN;
use super::reg::{RegErr, Registry};
use super::store_io;

impl Registry {
    pub fn register<D: crate::cap::Disk, R: crate::cap::Random>(
        &self,
        disk: &D,
        rng: &R,
        code: Vec<u8>,
    ) -> Result<Vec<u8>, RegErr<D::Err, (), R::Err>> {
        let mut pid = vec![0u8; ID_LEN];
        rng.fill(&mut pid).map_err(RegErr::Rand)?;
        store_io::put(disk, &self.dir, &pid, &code).map_err(RegErr::Store)?;
        self.code.write().unwrap().insert(pid.clone(), code);
        Ok(pid)
    }
    pub fn set<D: crate::cap::Disk>(
        &self,
        disk: &D,
        pid: &[u8],
        code: Vec<u8>,
    ) -> Result<(), RegErr<D::Err, (), ()>> {
        let mut map = self.code.write().unwrap();
        map.get(pid).ok_or(RegErr::Missing)?;
        store_io::put(disk, &self.dir, pid, &code).map_err(RegErr::Store)?;
        map.insert(pid.to_vec(), code);
        Ok(())
    }
    pub fn lookup(&self, pid: &[u8]) -> Result<Vec<u8>, RegErr<(), (), ()>> {
        self.code
            .read()
            .unwrap()
            .get(pid)
            .cloned()
            .ok_or(RegErr::Missing)
    }
    pub fn revoke<D: crate::cap::Disk>(
        &self,
        disk: &D,
        pid: &[u8],
    ) -> Result<(), RegErr<D::Err, (), ()>> {
        let mut map = self.code.write().unwrap();
        map.get(pid).ok_or(RegErr::Missing)?;
        store_io::del(disk, &self.dir, pid).map_err(RegErr::Store)?;
        map.remove(pid);
        Ok(())
    }
}

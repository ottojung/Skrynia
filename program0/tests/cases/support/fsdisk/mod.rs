mod io;
mod list;
use skrynia_program0::cap::Disk;
use std::path::PathBuf;
pub struct FsDisk(pub PathBuf);
impl FsDisk {
    pub fn p(&self, p: &str) -> PathBuf {
        self.0.join(p.trim_start_matches('/'))
    }
}
impl Disk for FsDisk {
    type Err = std::io::Error;
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, Self::Err> {
        io::read_all(self, p)
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), Self::Err> {
        io::write_atomic(self, p, b)
    }
    fn remove(&self, p: &str) -> Result<(), Self::Err> {
        io::remove(self, p)
    }
    fn list(&self, p: &str) -> Result<Vec<String>, Self::Err> {
        list::list(self, p)
    }
}

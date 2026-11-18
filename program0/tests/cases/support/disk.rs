use skrynia_program0::cap::Disk;
use std::{collections::BTreeMap, sync::Mutex};
#[derive(Default)]
pub struct MemDisk(Mutex<BTreeMap<String, Vec<u8>>>);
impl Disk for MemDisk {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.0.lock().unwrap().get(p).cloned())
    }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> {
        self.0.lock().unwrap().insert(p.into(), b.to_vec());
        Ok(())
    }
    fn remove(&self, p: &str) -> Result<(), ()> {
        self.0.lock().unwrap().remove(p);
        Ok(())
    }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        let p = format!("{}/", d.trim_end_matches('/'));
        Ok(self
            .0
            .lock()
            .unwrap()
            .keys()
            .filter_map(|k| k.strip_prefix(&p).map(str::to_string))
            .collect())
    }
}

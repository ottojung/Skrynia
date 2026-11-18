use skrynia_program0::cap::Disk;
use skrynia_program0::store_io::{get, put};
use skrynia_program0::store_ls::list_ids;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
struct MemDisk(Mutex<BTreeMap<String, Vec<u8>>>);

impl Disk for MemDisk {
    type Err = ();
    fn read_all(&self, path: &str) -> Result<Option<Vec<u8>>, ()> {
        Ok(self.0.lock().unwrap().get(path).cloned())
    }
    fn write_atomic(&self, path: &str, bytes: &[u8]) -> Result<(), ()> {
        self.0
            .lock()
            .unwrap()
            .insert(path.to_string(), bytes.to_vec());
        Ok(())
    }
    fn remove(&self, path: &str) -> Result<(), ()> {
        self.0.lock().unwrap().remove(path);
        Ok(())
    }
    fn list(&self, dir: &str) -> Result<Vec<String>, ()> {
        let p = format!("{}/", dir.trim_end_matches('/'));
        let g = self.0.lock().unwrap();
        Ok(g.keys()
            .filter_map(|k| k.strip_prefix(&p).map(|s| s.to_string()))
            .collect())
    }
}

#[test]
fn store_roundtrip_and_list() {
    let disk = MemDisk::default();
    let id = vec![7u8; 32];
    put(&disk, "reg", &id, b"wasm").unwrap();
    assert_eq!(get(&disk, "reg", &id).unwrap(), b"wasm");
    let ids = list_ids(&disk, "reg").unwrap();
    assert_eq!(ids, vec![id]);
}

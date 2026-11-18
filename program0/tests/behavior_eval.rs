use skrynia_program0::cap::{Disk, Eval, Random};
use skrynia_program0::reg::Registry;
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Default)]
struct MemDisk(Mutex<BTreeMap<String, Vec<u8>>>);
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
struct Rng(u8);
impl Random for Rng {
    type Err = ();
    fn fill(&self, o: &mut [u8]) -> Result<(), ()> {
        for b in o {
            *b = self.0;
        }
        Ok(())
    }
}
struct GuardEval;
impl Eval for GuardEval {
    type Err = &'static str;
    fn eval(&self, w: &[u8], _: &[u8]) -> Result<Vec<u8>, Self::Err> {
        if w == b"UNSAFE_IMPORT" {
            Err("unsafe import")
        } else {
            Ok(w.to_vec())
        }
    }
}

#[test]
fn unsafe_import_rejected() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let id = reg
        .register(&disk, &Rng(4), b"UNSAFE_IMPORT".to_vec())
        .unwrap();
    assert!(reg.invoke(&disk, &GuardEval, &id, b"").is_err());
}

#[test]
fn wrong_secret_cannot_set() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let _ = reg.register(&disk, &Rng(5), b"v1".to_vec()).unwrap();
    assert!(reg.set(&disk, &vec![0u8; 32], b"v2".to_vec()).is_err());
}

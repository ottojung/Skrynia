use skrynia_program0::cap::{Disk, Eval, Random};
use skrynia_program0::reg::Registry;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::thread;

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
struct TagEval;
impl Eval for TagEval {
    type Err = ();
    fn eval(&self, w: &[u8], _: &[u8]) -> Result<Vec<u8>, ()> {
        Ok(w.to_vec())
    }
}

#[test]
fn concurrent_invoke_sees_v1_or_v2() {
    let disk = Arc::new(MemDisk::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let id = reg.register(&*disk, &Rng(3), b"v1".to_vec()).unwrap();
    let mut hs = Vec::new();
    for _ in 0..4 {
        let (r, d, pid) = (reg.clone(), disk.clone(), id.clone());
        hs.push(thread::spawn(move || {
            for _ in 0..200 {
                let v = r.invoke(&*d, &TagEval, &pid, b"").unwrap();
                assert!(v == b"v1" || v == b"v2");
            }
        }));
    }
    reg.set(&*disk, &id, b"v2".to_vec()).unwrap();
    for h in hs {
        h.join().unwrap();
    }
    assert_eq!(reg.invoke(&*disk, &TagEval, &id, b"").unwrap(), b"v2");
}

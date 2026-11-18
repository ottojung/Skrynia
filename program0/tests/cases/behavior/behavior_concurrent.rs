use crate::cases::support::{MemDisk, Rng};
use skrynia_program0::cap::Eval;
use skrynia_program0::reg::Registry;
use std::sync::Arc;
use std::thread;

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

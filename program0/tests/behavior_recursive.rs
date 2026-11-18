use skrynia_program0::cap::{Disk, Eval, Random};
use skrynia_program0::reg::Registry;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct MemDisk(Mutex<BTreeMap<String, Vec<u8>>>);
impl Disk for MemDisk {
    type Err = ();
    fn read_all(&self, p: &str) -> Result<Option<Vec<u8>>, ()> { Ok(self.0.lock().unwrap().get(p).cloned()) }
    fn write_atomic(&self, p: &str, b: &[u8]) -> Result<(), ()> { self.0.lock().unwrap().insert(p.into(), b.to_vec()); Ok(()) }
    fn remove(&self, p: &str) -> Result<(), ()> { self.0.lock().unwrap().remove(p); Ok(()) }
    fn list(&self, d: &str) -> Result<Vec<String>, ()> {
        let p = format!("{}/", d.trim_end_matches('/'));
        Ok(self.0.lock().unwrap().keys().filter_map(|k| k.strip_prefix(&p).map(str::to_string)).collect())
    }
}
struct Rng(u8); impl Random for Rng { type Err = (); fn fill(&self, o: &mut [u8]) -> Result<(), ()> { for b in o { *b = self.0; } Ok(()) } }
struct SelfEval { reg: Arc<Registry>, disk: Arc<MemDisk>, me: Vec<u8> }
impl Eval for SelfEval { type Err = (); fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
    if w == b"self" && !i.is_empty() { self.reg.invoke(&*self.disk, self, &self.me, &i[1..]).map_err(|_| ()) }
    else { Ok(vec![i.len() as u8]) }
}}

#[test]
fn recursive_self_lookup() {
    let disk = Arc::new(MemDisk::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let id = reg.register(&*disk, &Rng(1), b"self".to_vec()).unwrap();
    let eval = SelfEval { reg: reg.clone(), disk: disk.clone(), me: id.clone() };
    assert_eq!(reg.invoke(&*disk, &eval, &id, b"abc").unwrap(), vec![0]);
}

struct CycleEval { reg: Arc<Registry>, disk: Arc<MemDisk>, a: Vec<u8>, b: Vec<u8> }
impl Eval for CycleEval { type Err = (); fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> {
    match w { b"A" => self.reg.invoke(&*self.disk, self, &self.b, i).map_err(|_| ()),
      b"B" if !i.is_empty() => self.reg.invoke(&*self.disk, self, &self.a, &i[1..]).map_err(|_| ()),
      _ => Ok(b"done".to_vec()) }
}}

#[test]
fn cyclic_dependency_lookup() {
    let disk = Arc::new(MemDisk::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let a = reg.register(&*disk, &Rng(2), b"A".to_vec()).unwrap();
    let b = reg.register(&*disk, &Rng(3), b"B".to_vec()).unwrap();
    let eval = CycleEval { reg: reg.clone(), disk: disk.clone(), a: a.clone(), b: b.clone() };
    assert_eq!(reg.invoke(&*disk, &eval, &a, b"xx").unwrap(), b"done");
}

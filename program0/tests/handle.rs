use skrynia_program0::cap::{Disk, Eval, Random};
use skrynia_program0::handle::handle;
use skrynia_program0::reg::Registry;
use skrynia_program0::wdec::decode_res;
use skrynia_program0::wenc::encode_req;
use skrynia_program0::wire::{Request, STATUS_OK, STATUS_ERR};
use std::collections::BTreeMap;
use std::sync::Mutex;

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
struct CatEval; impl Eval for CatEval { type Err = (); fn eval(&self, w: &[u8], i: &[u8]) -> Result<Vec<u8>, ()> { Ok([w, b":", i].concat()) } }

#[test]
fn wire_register_set_invoke() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let res = handle(&reg, &disk, &Rng(9), &CatEval, &encode_req(&Request::Register { code: b"a".to_vec() }));
    let res = decode_res(&res).unwrap();
    assert_eq!(res.status, STATUS_OK);
    let id = res.body;
    let out = handle(&reg, &disk, &Rng(9), &CatEval, &encode_req(&Request::Invoke { id: id.clone(), input: b"1".to_vec() }));
    assert_eq!(decode_res(&out).unwrap().body, b"a:1");
    handle(&reg, &disk, &Rng(9), &CatEval, &encode_req(&Request::Set { id: id.clone(), code: b"b".to_vec() }));
    let out = handle(&reg, &disk, &Rng(9), &CatEval, &encode_req(&Request::Invoke { id, input: b"2".to_vec() }));
    assert_eq!(decode_res(&out).unwrap().body, b"b:2");
}

#[test]
fn unknown_id_fails() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let out = handle(&reg, &disk, &Rng(1), &CatEval, &encode_req(&Request::Invoke { id: vec![0; 32], input: vec![] }));
    assert_eq!(decode_res(&out).unwrap().status, STATUS_ERR);
}

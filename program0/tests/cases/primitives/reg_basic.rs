use crate::cases::support::{MemDisk, Rng};
use skrynia_program0::cap::Eval;
use skrynia_program0::reg::Registry;

struct EchoEval;
impl Eval for EchoEval {
    type Err = ();
    fn eval(&self, wasm: &[u8], input: &[u8]) -> Result<Vec<u8>, ()> {
        Ok([wasm, input].concat())
    }
}

#[test]
fn register_set_invoke_and_recover() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let id = reg.register(&disk, &Rng(7), b"v1".to_vec()).unwrap();
    assert_eq!(reg.invoke(&disk, &EchoEval, &id, b"in").unwrap(), b"v1in");
    reg.set(&disk, &id, b"v2".to_vec()).unwrap();
    assert_eq!(reg.lookup(&id).unwrap(), b"v2");
    assert_eq!(reg.invoke(&disk, &EchoEval, &id, b"x").unwrap(), b"v2x");
    let reg2 = Registry::open(&disk, "reg").unwrap();
    assert_eq!(reg2.invoke(&disk, &EchoEval, &id, b"y").unwrap(), b"v2y");
    reg2.revoke(&disk, &id).unwrap();
    assert!(reg2.lookup(&id).is_err());
    assert!(reg2.set(&disk, &id, b"v3".to_vec()).is_err());
}

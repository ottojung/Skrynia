use crate::cases::support::{MemDisk, Rng};
use skrynia_program0::cap::Eval;
use skrynia_program0::reg::Registry;

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
    assert!(reg.set(&disk, &[0u8; 32], b"v2".to_vec()).is_err());
}

#[test]
fn wrong_secret_cannot_revoke_or_lookup() {
    let disk = MemDisk::default();
    let reg = Registry::open(&disk, "reg").unwrap();
    let _ = reg.register(&disk, &Rng(6), b"v1".to_vec()).unwrap();
    assert!(reg.revoke(&disk, &[0u8; 32]).is_err());
    assert!(reg.lookup(&[0u8; 32]).is_err());
    assert!(reg.invoke(&disk, &GuardEval, &[0u8; 32], b"").is_err());
}

impl Random for Rng {
    type Err = ();
    fn fill(&self, out: &mut [u8]) -> Result<(), ()> {
        for b in out {
            *b = self.0;
        }
        Ok(())
    }
}
struct Tag;
impl Eval for Tag {
    type Err = ();
    fn eval(&self, wasm: &[u8], _: &[u8]) -> Result<Vec<u8>, ()> {
        Ok(wasm.to_vec())
    }
}

#[test]
fn durable_replace_and_recover() {
    let dir = tmp("dur");
    let disk = Arc::new(FsDisk(dir.clone()));
    let reg = Registry::open(&*disk, "reg").unwrap();
    let id = reg.register(&*disk, &Rng(11), b"v1".to_vec()).unwrap();
    reg.set(&*disk, &id, b"v2".to_vec()).unwrap();
    let reg2 = Registry::open(&*disk, "reg").unwrap();
    assert_eq!(reg2.lookup(&id).unwrap(), b"v2");
    assert_eq!(reg2.invoke(&*disk, &Tag, &id, b"").unwrap(), b"v2");
    reg2.revoke(&*disk, &id).unwrap();
    let reg3 = Registry::open(&*disk, "reg").unwrap();
    assert!(reg3.lookup(&id).is_err());
    let _ = fs::remove_dir_all(dir);
}

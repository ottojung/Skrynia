use super::*;
#[test]
fn recursive_self_lookup() {
    let disk = Arc::new(MemDisk::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let id = reg.register(&*disk, &Rng(1), b"self".to_vec()).unwrap();
    let eval = SelfEval {
        reg: reg.clone(),
        disk: disk.clone(),
        me: id.clone(),
    };
    assert_eq!(reg.invoke(&*disk, &eval, &id, b"abc").unwrap(), vec![0]);
}

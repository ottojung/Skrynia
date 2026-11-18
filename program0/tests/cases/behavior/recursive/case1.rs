use super::*;
#[test]
fn cyclic_dependency_lookup() {
    let disk = Arc::new(MemDisk::default());
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let a = reg.register(&*disk, &Rng(2), b"A".to_vec()).unwrap();
    let b = reg.register(&*disk, &Rng(3), b"B".to_vec()).unwrap();
    let eval = CycleEval {
        reg: reg.clone(),
        disk: disk.clone(),
        a: a.clone(),
        b: b.clone(),
    };
    assert_eq!(reg.invoke(&*disk, &eval, &a, b"xx").unwrap(), b"done");
}

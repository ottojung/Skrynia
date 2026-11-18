use super::*;
#[test]
fn durable_concurrent_invoke_during_replace() {
    let dir = tmp("durc");
    let disk = Arc::new(FsDisk(dir.clone()));
    let reg = Arc::new(Registry::open(&*disk, "reg").unwrap());
    let id = reg.register(&*disk, &Rng(12), b"v1".to_vec()).unwrap();
    let mut hs = Vec::new();
    for _ in 0..4 {
        let (r, d, pid) = (reg.clone(), disk.clone(), id.clone());
        hs.push(thread::spawn(move || {
            for _ in 0..200 {
                let v = r.invoke(&*d, &Tag, &pid, b"").unwrap();
                assert!(v == b"v1" || v == b"v2");
            }
        }));
    }
    reg.set(&*disk, &id, b"v2".to_vec()).unwrap();
    for h in hs {
        h.join().unwrap();
    }
    assert_eq!(
        Registry::open(&*disk, "reg").unwrap().lookup(&id).unwrap(),
        b"v2"
    );
    let _ = fs::remove_dir_all(dir);
}

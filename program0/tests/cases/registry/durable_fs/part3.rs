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

#[test]
fn open_skips_corrupt_records() {
    let dir = tmp("corrupt");
    let disk = FsDisk(dir.clone());
    let reg = Registry::open(&disk, "reg").unwrap();
    let id = reg.register(&disk, &Rng(13), b"ok".to_vec()).unwrap();
    let regdir = dir.join("reg");
    fs::create_dir_all(&regdir).unwrap();
    fs::write(regdir.join("not-a-record.txt"), b"junk").unwrap();
    fs::write(regdir.join("00.bin"), b"bad").unwrap();
    let reg2 = Registry::open(&disk, "reg").unwrap();
    assert_eq!(reg2.lookup(&id).unwrap(), b"ok");
    let _ = fs::remove_dir_all(dir);
}

use super::*;
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

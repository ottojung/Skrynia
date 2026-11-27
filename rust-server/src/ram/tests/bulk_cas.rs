use super::Ram;
use std::sync::Arc;
use std::thread;

#[test]
fn bulk_writes_publish_through_cas() {
    let r = Arc::new(Ram::new());
    let flag = 4096;
    r.truncate(flag + 1).unwrap();
    r.write(0, &[0; 4096]).unwrap();
    r.write(flag, &[0]).unwrap();
    let w = r.clone();
    let t = thread::spawn(move || {
        w.write(0, &[0xab; 4096]).unwrap();
        assert_eq!(w.cas1(flag, 0, 1).unwrap(), 0);
    });
    while r.cas1(flag, 0, 0).unwrap() == 0 {
        thread::yield_now();
    }
    let mut dst = [0u8; 4096];
    assert_eq!(r.read(0, &mut dst).unwrap(), 4096);
    assert!(dst.iter().all(|b| *b == 0xab));
    assert_eq!(r.read(flag, &mut [0u8; 1]).unwrap(), 1);
    t.join().unwrap();
}

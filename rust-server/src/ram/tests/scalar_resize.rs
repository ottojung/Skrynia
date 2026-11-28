use super::Ram;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn scalar_ops_survive_concurrent_shrink_and_regrow() {
    let r = Arc::new(Ram::new());
    r.truncate(8192).unwrap();
    let ready = Arc::new(Barrier::new(2));
    let (w, start) = (r.clone(), ready.clone());
    let t = thread::spawn(move || {
        start.wait();
        for _ in 0..128 {
            w.truncate(4096).unwrap();
            w.truncate(8192).unwrap();
        }
    });
    ready.wait();
    for i in 0..2048u64 {
        let retained = i % 4096;
        let removed = retained + 4096;
        assert_eq!(r.write_byte(retained, 1), Ok(()));
        assert_eq!(r.cas1(retained, 1, 2), Ok(1));
        assert_eq!(r.read_byte(retained), Ok(2));
        assert!(matches!(r.write_byte(removed, 7), Ok(()) | Err("oob")));
        assert!(matches!(r.cas1(removed, 7, 8), Ok(0 | 7 | 8) | Err("oob")));
        assert!(matches!(r.read_byte(removed), Ok(0 | 7 | 8) | Err("oob")));
    }
    t.join().unwrap();
    assert_eq!(r.len(), 8192);
    assert_eq!(r.read_byte(0), Ok(2));
    r.truncate(4096).unwrap();
    r.truncate(8192).unwrap();
    assert_eq!(r.read_byte(4096), Ok(0));
}

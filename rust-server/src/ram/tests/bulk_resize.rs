use super::Ram;
use std::sync::Arc;
use std::thread;

#[test]
fn bulk_spans_survive_concurrent_shrink_and_regrow() {
    let r = Arc::new(Ram::new());
    r.truncate(64 * 1024).unwrap();
    let w = r.clone();
    let t = thread::spawn(move || {
        for _ in 0..64 {
            w.truncate(12_345).unwrap();
            w.truncate(64 * 1024).unwrap();
        }
    });
    let src = vec![0x7fu8; 16 * 1024];
    let mut dst = vec![0u8; src.len()];
    for _ in 0..32 {
        assert!(r.write(0, &src).is_ok());
        assert!(r.read(0, &mut dst).is_ok());
    }
    t.join().unwrap();
}

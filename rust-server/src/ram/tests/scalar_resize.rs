use super::Ram;
use std::sync::Arc;
use std::thread;

#[test]
fn scalar_ops_survive_concurrent_shrink_and_regrow() {
    let r = Arc::new(Ram::new());
    r.truncate(64 * 1024).unwrap();
    let w = r.clone();
    let t = thread::spawn(move || {
        for _ in 0..64 {
            w.truncate(4096).unwrap();
            w.truncate(64 * 1024).unwrap();
        }
    });
    for i in 0..1024u64 {
        let _ = r.read_byte(i);
        let _ = r.write_byte(i, 1);
        let _ = r.cas1(i, 1, 2);
    }
    t.join().unwrap();
}

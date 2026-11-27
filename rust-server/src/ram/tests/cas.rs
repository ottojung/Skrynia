use super::Ram;
use std::sync::Arc;
use std::thread;

#[test]
fn cas_publishes_prior_byte_writes() {
    let r = Arc::new(Ram::new());
    let flag = 4096;
    r.truncate(flag + 1).unwrap();
    // Preallocate distinct shards so lazy allocation cannot publish the data.
    r.write_byte(0, 0).unwrap();
    r.write_byte(flag, 0).unwrap();
    let w = r.clone();
    let t = thread::spawn(move || {
        w.write_byte(0, 0xab).unwrap();
        w.write_byte(1, 0xcd).unwrap();
        assert_eq!(w.cas1(flag, 0, 1).unwrap(), 0);
    });
    while r.cas1(flag, 0, 0).unwrap() == 0 {
        thread::yield_now();
    }
    // Read before join: the failed CAS must itself acquire the publication.
    assert_eq!(r.read_byte(0).unwrap(), 0xab);
    assert_eq!(r.read_byte(1).unwrap(), 0xcd);
    assert_eq!(r.read_byte(flag).unwrap(), 1);
    t.join().unwrap();
}

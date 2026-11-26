use super::Ram;
use std::sync::Arc;
use std::thread;

#[test]
fn truncate_grows_zeroes_and_shrinks() {
    let r = Ram::new();
    r.truncate(4).unwrap();
    r.write_byte(2, 0x7f).unwrap();
    assert_eq!(r.read_byte(2).unwrap(), 0x7f);
    r.truncate(8).unwrap();
    assert_eq!(r.read_byte(2).unwrap(), 0x7f);
    assert_eq!(r.read_byte(7).unwrap(), 0);
    r.truncate(3).unwrap();
    assert_eq!(r.len(), 3);
    assert!(r.read_byte(3).is_err());
    r.truncate(8).unwrap();
    assert_eq!(r.read_byte(3).unwrap(), 0);
    assert!(r.truncate(super::MAX_RAM_BYTES + 1).is_err());
}

#[test]
fn cas_publishes_prior_byte_writes() {
    let r = Arc::new(Ram::new());
    r.truncate(8).unwrap();
    let w = r.clone();
    let t = thread::spawn(move || {
        w.write_byte(0, 0xab).unwrap();
        w.write_byte(1, 0xcd).unwrap();
        assert_eq!(w.cas1(7, 0, 1).unwrap(), 0);
    });
    while r.cas1(7, 0, 0).unwrap() == 0 {
        thread::yield_now();
    }
    t.join().unwrap();
    assert_eq!(r.read_byte(0).unwrap(), 0xab);
    assert_eq!(r.read_byte(1).unwrap(), 0xcd);
    assert_eq!(r.read_byte(7).unwrap(), 1);
}

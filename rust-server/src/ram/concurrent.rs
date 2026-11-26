use super::Ram;
use std::sync::Arc;
use std::thread;

#[test]
fn disjoint_writes_are_not_lost() {
    let r = Arc::new(Ram::new());
    r.truncate(4096).unwrap();
    let mut hs = Vec::new();
    for t in 0..4u64 {
        let w = r.clone();
        hs.push(thread::spawn(move || {
            for i in 0..1024u64 {
                w.write_byte(t * 1024 + i, (t + i) as u8).unwrap();
            }
        }));
    }
    for h in hs {
        h.join().unwrap();
    }
    for t in 0..4u64 {
        for i in 0..1024u64 {
            assert_eq!(r.read_byte(t * 1024 + i).unwrap(), (t + i) as u8);
        }
    }
}

#[test]
fn concurrent_truncate_is_host_safe() {
    let r = Arc::new(Ram::new());
    r.truncate(1024).unwrap();
    let w = r.clone();
    let t = thread::spawn(move || {
        for n in [512, 2048, 256, 1024] {
            w.truncate(n).unwrap();
        }
    });
    for i in 0..1024u64 {
        let _ = r.write_byte(i, 1);
        let _ = r.cas1(i, 1, 2);
        let _ = r.read_byte(i);
    }
    t.join().unwrap();
}

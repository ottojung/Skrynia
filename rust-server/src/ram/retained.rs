use super::Ram;
use std::sync::{Arc, Barrier};
use std::thread;

#[test]
fn retained_bytes_survive_overlapping_resizes() {
    let r = Arc::new(Ram::new());
    let stable = 128 * 1024;
    r.truncate(stable).unwrap();
    let start = Arc::new(Barrier::new(3));
    thread::scope(|scope| {
        let begin = start.clone();
        let ram = r.clone();
        scope.spawn(move || {
            begin.wait();
            for _ in 0..64 {
                ram.truncate(stable * 2).unwrap();
                ram.truncate(stable).unwrap();
            }
        });
        let begin = start.clone();
        let ram = r.clone();
        scope.spawn(move || {
            begin.wait();
            for i in 0..stable / 2 {
                ram.write_byte(i, 37).unwrap();
            }
        });
        start.wait();
        for i in stable / 2..stable {
            assert_eq!(r.cas1(i, 0, 129), Ok(0));
        }
    });
    for i in 0..stable {
        assert_eq!(
            r.read_byte(i),
            Ok(if i < stable / 2 { 37 } else { 129 }),
            "offset {i}"
        );
    }
}

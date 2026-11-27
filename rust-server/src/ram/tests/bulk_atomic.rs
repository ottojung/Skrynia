use super::Ram;
use std::sync::Arc;
use std::thread;

#[test]
fn disjoint_bulk_writes_keep_their_bytes() {
    let r = Arc::new(Ram::new());
    r.truncate(4096).unwrap();
    let writers: Vec<_> = (0..4u64)
        .map(|t| {
            let w = r.clone();
            thread::spawn(move || {
                let src = vec![t as u8 + 1; 1024];
                assert_eq!(w.write(t * 1024, &src), Ok(1024));
            })
        })
        .collect();
    for w in writers {
        w.join().unwrap();
    }
    for t in 0..4u64 {
        let mut dst = vec![0u8; 1024];
        assert_eq!(r.read(t * 1024, &mut dst), Ok(1024));
        assert!(dst.iter().all(|&b| b == t as u8 + 1));
    }
}

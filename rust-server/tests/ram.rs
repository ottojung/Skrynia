use skrynia_server::ram::Ram;
use std::sync::Arc;

#[test]
fn read_write_and_bounds() {
    let ram = Ram::new();
    ram.truncate(8).unwrap();
    assert_eq!(ram.write(0, &[1, 2, 3, 4]).unwrap(), 4);
    let mut buf = [0u8; 4];
    assert_eq!(ram.read(0, &mut buf).unwrap(), 4);
    assert_eq!(buf, [1, 2, 3, 4]);
    assert!(ram.read(9, &mut buf).is_err());
    assert!(ram.write(9, &[1]).is_err());
}

#[test]
fn truncate_zero_fills() {
    let ram = Ram::new();
    ram.truncate(4).unwrap();
    ram.write(0, &[9, 9, 9, 9]).unwrap();
    ram.truncate(8).unwrap();
    let mut buf = [1u8; 8];
    ram.read(0, &mut buf).unwrap();
    assert_eq!(&buf[4..], &[0, 0, 0, 0]);
}

#[test]
fn cas_alignment_bounds_and_values() {
    let ram = Ram::new();
    ram.truncate(16).unwrap();
    assert!(ram.cas_u64(1, 0, 1).is_err());
    assert!(ram.cas_u64(16, 0, 1).is_err());
    assert_eq!(ram.cas_u64(0, 0, 7).unwrap(), 0);
    assert_eq!(ram.cas_u64(0, 0, 9).unwrap(), 7);
    assert_eq!(ram.cas_u64(0, 7, 9).unwrap(), 7);
}

#[test]
fn cas_has_one_winner() {
    let ram = Arc::new(Ram::new());
    ram.truncate(8).unwrap();
    let mut hs = Vec::new();
    for i in 1u64..=8 {
        let r = ram.clone();
        hs.push(std::thread::spawn(move || r.cas_u64(0, 0, i).unwrap()));
    }
    let olds: Vec<u64> = hs.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(olds.iter().filter(|&&v| v == 0).count(), 1);
}

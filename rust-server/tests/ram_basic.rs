use skrynia_server::ram::Ram;

#[test]
fn read_write_and_bounds() {
    let ram = Ram::new();
    ram.truncate(8);
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
    ram.truncate(4);
    ram.write(0, &[9, 9, 9, 9]).unwrap();
    ram.truncate(8);
    let mut buf = [1u8; 8];
    ram.read(0, &mut buf).unwrap();
    assert_eq!(&buf[4..], &[0, 0, 0, 0]);
}

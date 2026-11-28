use super::Ram;

#[test]
fn removed_pages_read_zero_after_regrow_scalar() {
    let r = Ram::new();
    r.truncate(8192).unwrap();
    r.write_byte(4096, 0xAB).unwrap();
    r.write_byte(5000, 0xCD).unwrap();
    r.truncate(4096).unwrap();
    r.truncate(8192).unwrap();
    assert_eq!(r.read_byte(4096), Ok(0));
    assert_eq!(r.read_byte(5000), Ok(0));
    assert_eq!(r.read_byte(0), Ok(0));
}

#[test]
fn retained_page_tail_shrink_regrow_scalar() {
    let r = Ram::new();
    r.truncate(8192).unwrap();
    r.write_byte(4095, 0x11).unwrap();
    r.write_byte(4096, 0x22).unwrap();
    r.write_byte(4100, 0x33).unwrap();
    r.truncate(4097).unwrap();
    assert_eq!(r.read_byte(4095), Ok(0x11));
    assert_eq!(r.read_byte(4096), Ok(0x22));
    assert!(r.read_byte(4097).is_err());
    r.truncate(8192).unwrap();
    assert_eq!(r.read_byte(4095), Ok(0x11));
    assert_eq!(r.read_byte(4096), Ok(0x22));
    assert_eq!(r.read_byte(4100), Ok(0));
}

#[test]
fn removed_pages_read_zero_after_regrow_bulk() {
    let r = Ram::new();
    r.truncate(8192).unwrap();
    r.write(4096, &[1, 2, 3, 4, 5]).unwrap();
    r.truncate(4096).unwrap();
    r.truncate(8192).unwrap();
    let mut dst = [9u8; 8];
    assert_eq!(r.read(4096, &mut dst), Ok(8));
    assert_eq!(dst, [0; 8]);
}

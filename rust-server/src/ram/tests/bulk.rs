use super::Ram;

#[test]
fn read_eof_zero_and_partial_progress() {
    let r = Ram::new();
    r.truncate(5).unwrap();
    r.write(0, b"abcde").unwrap();
    let mut dst = [0xffu8; 5];
    assert_eq!(r.read(4, &mut dst), Ok(1));
    assert_eq!(dst[0], b'e');
    assert_eq!(r.read(5, &mut dst), Ok(0));
    assert_eq!(r.read(99, &mut dst), Ok(0));
    assert_eq!(r.read(5, &mut []), Ok(0));
    r.truncate(2).unwrap();
    assert_eq!(r.read(2, &mut dst), Ok(0));
    assert_eq!(r.read(9, &mut dst), Ok(0));
    assert_eq!(r.read(1, &mut dst), Ok(1));
    assert_eq!(dst[0], b'b');
    r.truncate(5).unwrap();
    assert_eq!(r.read(2, &mut dst), Ok(3));
    assert_eq!(&dst[..3], &[0, 0, 0]);
}

#[test]
fn write_bounds_zero_length_and_partial_progress() {
    let r = Ram::new();
    r.truncate(4).unwrap();
    assert_eq!(r.write(4, b"x"), Err("oob"));
    assert_eq!(r.write(9, b"x"), Err("oob"));
    assert_eq!(r.write(4, b""), Ok(0));
    assert_eq!(r.write(9, b""), Ok(0));
    assert_eq!(r.write(2, b"abcd"), Ok(2));
    let mut dst = [0u8; 4];
    assert_eq!(r.read(0, &mut dst), Ok(4));
    assert_eq!(&dst, &[0, 0, b'a', b'b']);
}

#[test]
fn cross_page_roundtrip() {
    let r = Ram::new();
    r.truncate(4 * 4096).unwrap();
    let src = vec![0x5a; 2 * 4096 + 7];
    assert_eq!(r.write(4094, &src), Ok(src.len()));
    let mut dst = vec![0u8; src.len()];
    assert_eq!(r.read(4094, &mut dst), Ok(src.len()));
    assert_eq!(dst, src);
}

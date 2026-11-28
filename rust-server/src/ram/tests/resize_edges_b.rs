use super::Ram;

#[test]
fn sparse_read_and_cas_edges() {
    let r = Ram::new();
    let base = (1u64 << 33) + 10;
    r.truncate(base + 64).unwrap();
    assert_eq!(r.read_byte(base), Ok(0));
    assert_eq!(r.cas1(base, 0, 7), Ok(0));
    assert_eq!(r.read_byte(base), Ok(7));
    assert_eq!(r.cas1(base, 0, 9), Ok(7));
    assert_eq!(r.read_byte(base), Ok(7));
    r.truncate(base + 8).unwrap();
    assert!(r.read_byte(base + 8).is_err());
    r.truncate(base + 64).unwrap();
    assert_eq!(r.read_byte(base + 8), Ok(0));
}

#[test]
fn scalar_ops_error_on_removed_range_after_shrink() {
    let r = Ram::new();
    r.truncate(100).unwrap();
    r.write_byte(50, 5).unwrap();
    r.truncate(10).unwrap();
    assert!(r.read_byte(50).is_err());
    assert!(r.write_byte(50, 1).is_err());
    assert!(r.cas1(50, 0, 1).is_err());
}

use skrynia_server::ram::Ram;

#[test]
fn cas_alignment_bounds_and_values() {
    let ram = Ram::new();
    ram.truncate(16);
    assert!(ram.cas_u64(1, 0, 1).is_err());
    assert!(ram.cas_u64(16, 0, 1).is_err());
    assert_eq!(ram.cas_u64(0, 0, 7).unwrap(), 0);
    assert_eq!(ram.cas_u64(0, 0, 9).unwrap(), 7);
    assert_eq!(ram.cas_u64(0, 7, 9).unwrap(), 7);
}

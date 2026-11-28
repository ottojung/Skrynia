use super::Ram;
mod bench;
mod bulk;
mod bulk_atomic;
mod bulk_cas;
mod bulk_resize;
mod cas;
mod scalar_resize;

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
fn sparse_ram_supports_offsets_beyond_four_gib() {
    let r = Ram::new();
    let at = 5u64 << 30;
    r.truncate(at + 2).unwrap();
    assert_eq!(r.read_byte(at).unwrap(), 0);
    assert_eq!(r.cas1(at, 0, 73).unwrap(), 0);
    assert_eq!(r.read_byte(at).unwrap(), 73);
    r.truncate(at).unwrap();
    r.truncate(at + 2).unwrap();
    assert_eq!(r.read_byte(at).unwrap(), 0);
}

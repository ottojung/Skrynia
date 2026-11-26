use libskrynia::fs::table::Table;
#[test]
fn fd_generations_never_alias_or_turn_into_errors() {
    let mut table = Table::new();
    let stale = table.insert(10u32).unwrap();
    let mut fd = stale;
    for n in 1..=0x7fff {
        assert!(fd.0 > 0, "valid fd must not overlap negative error codes");
        assert_eq!(table.remove(fd), Some(10));
        assert!(table.get(stale).is_none());
        fd = table.insert(10).unwrap();
        if n < 0x7fff {
            assert_eq!(fd.idx(), stale.idx());
        }
    }
    assert_ne!(fd.idx(), stale.idx(), "retired slot must not wrap");
    assert!(fd.0 > 0);
    assert!(table.get(stale).is_none());
}

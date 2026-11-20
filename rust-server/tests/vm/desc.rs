use skrynia_server::fs::table::Table;

#[test]
fn fd_table_limits_and_targets() {
    let mut t = Table::new();
    let mut fds = Vec::with_capacity(1 << 16);
    for i in 0u32..(1 << 16) {
        let fd = t.insert(i).unwrap();
        fds.push(fd);
        assert_eq!(*t.get(fd).unwrap(), i);
    }
    assert!(t.insert(99).is_err());
    for (i, fd) in fds.iter().copied().enumerate().take(1024) {
        assert_eq!(*t.get(fd).unwrap(), i as u32);
    }
    let victim = fds[1234];
    assert_eq!(t.remove(victim), Some(1234));
    let reused = t.insert(7777).unwrap();
    assert_ne!(reused, victim);
    assert_eq!(*t.get(reused).unwrap(), 7777);
    assert!(t.get(victim).is_none());
}

#[test]
fn fd_generations_never_alias_or_turn_into_errors() {
    let mut table = Table::new();
    let stale = table.insert(10u32).unwrap();
    let mut fd = stale;
    for n in 1..=0x7fff {
        assert!(fd.0 > 0);
        assert_eq!(table.remove(fd), Some(10));
        assert!(table.get(stale).is_none());
        fd = table.insert(10).unwrap();
        if n < 0x7fff {
            assert_eq!(fd.idx(), stale.idx());
        }
    }
    assert_ne!(fd.idx(), stale.idx());
    assert!(fd.0 > 0);
    assert!(table.get(stale).is_none());
}

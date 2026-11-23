use skrynia_server::fs::table::Table;

#[test]
fn fd_table_limits_and_targets() {
    let mut t = Table::new();
    let mut fds = Vec::with_capacity(1 << 16);
    for i in 0u32..(1 << 16) {
        let fd = t.insert(i, 0).unwrap();
        fds.push(fd);
        assert_eq!(*t.get(fd).unwrap(), i);
    }
    assert!(t.insert(99, 0).is_err());
    for (i, fd) in fds.iter().copied().enumerate().take(1024) {
        assert_eq!(*t.get(fd).unwrap(), i as u32);
    }
    let victim = fds[1234];
    assert_eq!(t.remove(victim), Some(1234));
    let reused = t.insert(7777, 0).unwrap();
    assert_ne!(reused, victim);
    assert_eq!(*t.get(reused).unwrap(), 7777);
    assert!(t.get(victim).is_none());
    assert_eq!(t.remove(reused), Some(7777));
    for i in 0..4096usize {
        let fd = t.insert(i as u32, 0).unwrap();
        assert_eq!(t.remove(fd), Some(i as u32));
    }
}

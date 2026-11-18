use skrynia_server::ram::Ram;
use std::sync::Arc;

#[test]
fn cas_has_one_winner() {
    let ram = Arc::new(Ram::new());
    ram.truncate(8);
    let mut handles = Vec::new();
    for i in 1u64..=8 {
        let r = ram.clone();
        handles.push(std::thread::spawn(move || r.cas_u64(0, 0, i).unwrap()));
    }
    let olds: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    assert_eq!(olds.iter().filter(|&&v| v == 0).count(), 1);
    assert!(olds.contains(&(olds.iter().copied().find(|&v| v != 0).unwrap())));
}

use super::guard::Upload;
use std::sync::Arc;
use tokio::sync::Semaphore;
#[test]
fn memory_reservations_are_bounded_and_released() {
    let sem = Arc::new(Semaphore::new(5));
    let mut first = Upload::empty(sem.clone());
    let mut second = Upload::empty(sem.clone());
    first.append(&[1; 4]).unwrap();
    assert!(second.append(&[2; 2]).is_err());
    drop(first);
    assert!(second.append(&[3; 2]).is_ok());
    assert_eq!(second.bytes, [3, 3]);
    drop(second);
    assert_eq!(sem.available_permits(), 5);
}

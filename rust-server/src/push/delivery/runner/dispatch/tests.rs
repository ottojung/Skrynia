use super::run;
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[tokio::test]
async fn ready_work_continues_while_an_earlier_delivery_is_slow() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let work = (0..3).map(|id| {
        let events = Arc::clone(&events);
        async move {
            tokio::time::sleep(Duration::from_millis(if id == 0 { 100 } else { 5 })).await;
            events.lock().unwrap().push(id);
        }
    });
    assert!(run(work, 2, |_| async { true }).await);
    assert_eq!(*events.lock().unwrap(), vec![1, 2, 0]);
}
#[tokio::test]
async fn worker_panics_are_reported_without_discarding_other_work() {
    let work = (0..3).map(|id| async move {
        if id == 1 {
            panic!("simulated worker panic");
        }
    });
    assert!(!run(work, 2, |_| async { true }).await);
}

mod check;
mod flush;

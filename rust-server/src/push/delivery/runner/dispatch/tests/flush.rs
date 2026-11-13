use super::run;
use crate::{persist_outbox, read_outbox};
use std::{
    fs,
    sync::{Arc, Mutex},
};
#[tokio::test]
async fn fast_deliveries_are_persisted_before_a_slow_delivery_finishes() {
    let root = std::env::temp_dir().join(format!("skrynia-flush-{}", crate::random_id()));
    fs::create_dir_all(root.join("state/push")).unwrap();
    let original = super::check::pending();
    persist_outbox(&root, &original).unwrap();
    let pending = Arc::new(Mutex::new(original));
    let slow_done = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let work = (0..11).map(|index| {
        let done = Arc::clone(&slow_done);
        async move {
            if index == 10 {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                done.store(true, std::sync::atomic::Ordering::Release);
            }
            index
        }
    });
    let flushing = run(work, 11, |ids| {
        let pending = Arc::clone(&pending);
        let root = root.clone();
        async move {
            let mut records = pending.lock().unwrap();
            for i in ids {
                records.remove(&format!("{i:032x}"));
            }
            persist_outbox(&root, &records).is_ok()
        }
    });
    let checking = super::check::before_slow(&root, &slow_done);
    let (result, ()) = tokio::join!(flushing, checking);
    assert!(result);
    assert!(read_outbox(&root).unwrap().is_empty());
    fs::remove_dir_all(root).unwrap();
}

use crate::read_outbox;
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
pub async fn before_slow(root: &Path, slow_done: &Arc<AtomicBool>) {
    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
    let records = read_outbox(root).unwrap();
    assert_eq!(records.len(), 1);
    assert!(records.contains_key(&format!("{:032x}", 10)));
    assert!(!slow_done.load(Ordering::Acquire));
}

pub fn pending() -> std::collections::HashMap<String, crate::Pending> {
    (0..11)
        .map(|i| {
            (
                format!("{i:032x}"),
                crate::Pending {
                    generation: 1,
                    attempts: 0,
                    next_at: 0,
                },
            )
        })
        .collect()
}

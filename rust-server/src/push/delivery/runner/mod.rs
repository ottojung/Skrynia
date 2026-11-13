mod batch;
mod select;
use crate::State;
use std::sync::Arc;

pub async fn run(state: Arc<State>) {
    let mut timer = tokio::time::interval(std::time::Duration::from_secs(1));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut ticks = 0u32;
    loop {
        timer.tick().await;
        if state.failed.load(std::sync::atomic::Ordering::Acquire) {
            return;
        }
        if ticks.is_multiple_of(60) {
            let recovering = Arc::clone(&state);
            let result = tokio::task::spawn_blocking(move || recovering.recover_journal()).await;
            if let Err(error) = result.unwrap_or_else(|e| Err(std::io::Error::other(e))) {
                eprintln!("Skrynia pending notification recovery: {error}");
                return;
            }
        }
        ticks = ticks.wrapping_add(1);
        let entries = select::due(&state);
        for chunk in entries.chunks(state.push_config.concurrency) {
            batch::process(&state, chunk).await;
        }
    }
}

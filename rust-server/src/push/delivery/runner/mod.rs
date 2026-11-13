mod deliver;
mod dispatch;
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
        let snapshot = Arc::clone(&state);
        let entries = match tokio::task::spawn_blocking(move || select::due(&snapshot)).await {
            Ok(entries) => entries,
            Err(error) => {
                eprintln!("Skrynia notification selection failed: {error}");
                return;
            }
        };
        let work = entries.into_iter().map(|entry| {
            let state = Arc::clone(&state);
            async move { deliver::process(state, entry).await }
        });
        if !dispatch::run(work, state.push_config.concurrency).await {
            eprintln!("Skrynia push delivery worker panicked");
            state
                .failed
                .store(true, std::sync::atomic::Ordering::Release);
            return;
        }
    }
}

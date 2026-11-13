use super::select::Entry;
use crate::{PushFailure, State};
use std::sync::Arc;
pub async fn process(state: &Arc<State>, entries: &[Entry]) {
    let mut tasks = tokio::task::JoinSet::new();
    for (notification, subscription) in entries.iter().cloned() {
        let state = Arc::clone(state);
        tasks.spawn(async move {
            let outcome = if let Some(ref sub) = subscription {
                tokio::time::timeout(
                    std::time::Duration::from_millis(state.push_config.timeout_ms),
                    crate::push_send(sub, &state.push_config, &state.vapid),
                )
                .await
                .unwrap_or(Err(PushFailure::Retry))
            } else {
                Ok(())
            };
            let persisted = Arc::clone(&state);
            let saved = tokio::task::spawn_blocking(move || {
                crate::push_finish(&persisted, &notification, subscription.as_ref(), outcome)
            })
            .await;
            if let Err(error) = saved.unwrap_or_else(|error| Err(std::io::Error::other(error))) {
                eprintln!("Skrynia push persistence failure: {error}");
                state
                    .failed
                    .store(true, std::sync::atomic::Ordering::Release);
            }
        });
    }
    while let Some(result) = tasks.join_next().await {
        if let Err(error) = result {
            eprintln!("Skrynia push worker failure: {error}");
            state
                .failed
                .store(true, std::sync::atomic::Ordering::Release);
        }
    }
}

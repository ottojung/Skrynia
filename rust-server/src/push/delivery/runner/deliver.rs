use super::select::Entry;
use crate::{PushFailure, State};
use std::sync::{Arc, atomic::Ordering};
pub async fn process(state: Arc<State>, (notification, subscription): Entry) {
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
    let saving = Arc::clone(&state);
    let result = tokio::task::spawn_blocking(move || {
        crate::push_finish(&saving, &notification, subscription.as_ref(), outcome)
    })
    .await;
    if let Err(error) = result.unwrap_or_else(|error| Err(std::io::Error::other(error))) {
        eprintln!("Skrynia push persistence failure: {error}");
        state.failed.store(true, Ordering::Release);
    }
}

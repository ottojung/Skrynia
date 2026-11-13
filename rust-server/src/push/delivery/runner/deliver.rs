use super::select::Entry;
use crate::{DeliveryOutcome, PushFailure, State};
use std::sync::Arc;
pub async fn process(state: Arc<State>, (notification, subscription): Entry) -> DeliveryOutcome {
    let result = if let Some(ref sub) = subscription {
        tokio::time::timeout(
            std::time::Duration::from_millis(state.push_config.timeout_ms),
            crate::push_send(sub, &state.push_config, &state.vapid),
        )
        .await
        .unwrap_or(Err(PushFailure::Retry))
    } else {
        Ok(())
    };
    (notification, subscription, result)
}

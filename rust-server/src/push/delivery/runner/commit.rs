use crate::{DeliveryOutcome, State};
use std::{
    io,
    sync::{Arc, atomic::Ordering},
};
pub async fn completed(state: &Arc<State>, outcomes: Vec<DeliveryOutcome>) -> bool {
    let writer = Arc::clone(state);
    let result = tokio::task::spawn_blocking(move || crate::push_finish_batch(&writer, outcomes))
        .await
        .unwrap_or_else(|error| Err(io::Error::other(error)));
    if let Err(error) = result {
        eprintln!("Skrynia push persistence failure: {error}");
        state.failed.store(true, Ordering::Release);
        false
    } else {
        true
    }
}

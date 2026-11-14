mod ack;
mod apply;
mod fail;
#[cfg(test)]
mod fixture;
mod retire;
#[cfg(test)]
mod tests;
use crate::{Pending, PushFailure, State, Subscription};
use std::{io, sync::atomic::Ordering};
pub type DeliveryOutcome = (
    String,
    Pending,
    Option<Subscription>,
    Result<(), PushFailure>,
);
pub fn batch(state: &State, results: Vec<DeliveryOutcome>) -> io::Result<()> {
    let mut data = state
        .data
        .lock()
        .map_err(|_| io::Error::other("poisoned"))?;
    if state.failed.load(Ordering::Acquire) {
        return Err(io::Error::other("recovery required"));
    }
    let before = data.push.outbox.clone();
    let mut retired = Vec::new();
    let mut changed = false;
    for result in results {
        changed |= apply::change(state, &mut data, result, &mut retired);
    }
    let mut clean_persist_failure = false;
    let result = (|| {
        if changed {
            if let Err(error) = crate::persist_outbox(&state.dir, &data.push.outbox) {
                data.push.outbox = before;
                clean_persist_failure = crate::store::disk::clean_write_error(&error);
                return Err(error);
            }
        }
        for id in retired {
            retire::remove(state, &mut data, &id)?;
        }
        Ok(())
    })();
    if result.is_err() && !clean_persist_failure {
        state.failed.store(true, Ordering::Release);
    }
    result
}

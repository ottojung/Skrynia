mod ack;
mod fail;
#[cfg(test)]
mod fixture;
mod retire;
#[cfg(test)]
mod tests;
use crate::{Notification, PushFailure, State, Subscription};
use std::io;
use std::sync::atomic::Ordering;
pub fn finish(
    state: &State,
    original: &Notification,
    sub: Option<&Subscription>,
    outcome: Result<(), PushFailure>,
) -> io::Result<()> {
    let mut data = state
        .data
        .lock()
        .map_err(|_| io::Error::other("poisoned"))?;
    if state.failed.load(Ordering::Acquire) {
        return Err(io::Error::other("recovery required"));
    }
    let Some(current) = data.push.outbox.get(&original.subscription_id).cloned() else {
        return Ok(());
    };
    let persisted = if sub.is_none() || outcome.is_ok() {
        if sub.is_none() || current.generation == original.generation {
            ack::remove(&state.dir, &mut data, &original.subscription_id)
        } else {
            Ok(())
        }
    } else {
        fail::retry(
            state,
            &mut data,
            &current,
            original,
            sub.unwrap(),
            outcome.unwrap_err(),
        )
    };
    // A failing ack must freeze subsequent completions before unlocking.
    if persisted.is_err() {
        state.failed.store(true, Ordering::Release);
    }
    persisted
}

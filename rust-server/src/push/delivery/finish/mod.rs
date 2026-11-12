mod ack;
mod fail;
#[cfg(test)]
mod fixture;
mod retire;
#[cfg(test)]
mod test_retry;
#[cfg(test)]
mod test_success;
#[cfg(test)]
mod test_terminal;
use crate::{Notification, PushFailure, State, Subscription};
use std::io;
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
    let Some(current) = data.push.outbox.get(&original.subscription_id).cloned() else {
        return Ok(());
    };
    if sub.is_none() || outcome.is_ok() {
        if sub.is_none() || current.generation == original.generation {
            ack::remove(&state.dir, &mut data, &original.subscription_id)?;
        }
        return Ok(());
    }
    fail::retry(
        state,
        &mut data,
        &current,
        original,
        sub.unwrap(),
        outcome.unwrap_err(),
    )
}

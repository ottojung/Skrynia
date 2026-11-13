use crate::{Notification, PushFailure, State, Subscription};
use std::io;
pub fn finish(
    state: &State,
    original: &Notification,
    sub: Option<&Subscription>,
    outcome: Result<(), PushFailure>,
) -> io::Result<()> {
    super::batch(state, vec![(original.clone(), sub.cloned(), outcome)])
}

pub fn idle(state: &State) -> io::Result<()> {
    if state.failed.load(std::sync::atomic::Ordering::Acquire) {
        Err(io::Error::other("recovery required"))
    } else {
        Ok(())
    }
}

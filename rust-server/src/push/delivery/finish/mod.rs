mod ack;
mod apply;
mod fail;
#[cfg(test)]
pub(crate) mod fixture;
mod flush;
#[cfg(test)]
pub(crate) use fixture::Fixture;
mod report;
mod retire;
pub use report::DeliveryOutcome;
#[cfg(test)]
mod tests;
use crate::State;
use std::{io, sync::atomic::Ordering};
pub fn batch(state: &State, results: Vec<DeliveryOutcome>) -> io::Result<()> {
    let mut data = state
        .data
        .lock()
        .map_err(|_| io::Error::other("poisoned"))?;
    if state.failed.load(Ordering::Acquire) {
        return Err(io::Error::other("recovery required"));
    }
    flush::run(state, &mut data, results)
}

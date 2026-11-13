use crate::{Fail, State};
use std::sync::atomic::Ordering;
pub fn map(state: &State, failure: crate::store::journal::CommitFailure) -> Fail {
    let unsafe_state = matches!(failure, crate::store::journal::CommitFailure::Uncertain(_));
    let error = failure.error();
    if unsafe_state {
        state.failed.store(true, Ordering::Release);
        eprintln!("Skrynia storage recovery required: {error}");
        crate::internal()
    } else {
        crate::error::transient_io(error)
    }
}

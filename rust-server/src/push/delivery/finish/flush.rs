use super::{DeliveryOutcome, apply, retire};
use crate::{Snapshot, State};
use std::{io, sync::atomic::Ordering};
pub fn run(state: &State, data: &mut Snapshot, results: Vec<DeliveryOutcome>) -> io::Result<()> {
    let before = data.push.outbox.clone();
    let mut retired = Vec::new();
    let mut touched = Vec::new();
    for result in results {
        let id = result.0.clone();
        let delivery_failed = result.3.is_err();
        if apply::change(state, data, result, &mut retired) {
            touched.push((id, delivery_failed));
        }
    }
    if !touched.is_empty() {
        if let Err(error) = crate::persist_outbox(&state.dir, &data.push.outbox) {
            data.push.outbox = before;
            if crate::store::disk::clean_write_error(&error) {
                for (id, failed) in touched {
                    data.push.delay_failed_ack(&id, failed);
                }
            } else {
                state.failed.store(true, Ordering::Release);
            }
            return Err(error);
        }
        for (id, _) in touched {
            data.push.clear_cooldown(&id);
        }
    }
    for id in retired {
        if let Err(error) = retire::remove(state, data, &id) {
            state.failed.store(true, Ordering::Release);
            return Err(error);
        }
    }
    Ok(())
}

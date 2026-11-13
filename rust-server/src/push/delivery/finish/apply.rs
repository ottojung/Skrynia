use super::{DeliveryOutcome, ack, fail};
use crate::{Snapshot, State};
pub fn change(
    state: &State,
    data: &mut Snapshot,
    report: DeliveryOutcome,
    retired: &mut Vec<String>,
) -> bool {
    let (original, subscription, outcome) = report;
    let Some(current) = data.push.outbox.get(&original.subscription_id).cloned() else {
        return false;
    };
    match (subscription, outcome) {
        (Some(sub), Err(failure)) => {
            if fail::retry(state, data, &current, &original, &sub, failure) {
                retired.push(sub.id);
            }
            true
        }
        (sub, _) => {
            if sub.is_none() || current.generation == original.generation {
                ack::remove(data, &original.subscription_id);
                true
            } else {
                false
            }
        }
    }
}

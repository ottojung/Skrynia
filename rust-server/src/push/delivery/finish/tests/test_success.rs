use super::Fixture;
use crate::{Pending, PushFailure, push_finish_batch};
#[test]
fn newer_generation_survives_stale_acknowledgment() {
    let f = Fixture::new(2);
    let stale = Pending {
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    push_finish_batch(
        &f.state,
        vec![(f.sub.id.clone(), stale, Some(f.sub.clone()), Ok(()))],
    )
    .unwrap();
    assert_eq!(f.read().generation, 2);
    let latest = f.read();
    push_finish_batch(
        &f.state,
        vec![(f.sub.id.clone(), latest, Some(f.sub.clone()), Ok(()))],
    )
    .unwrap();
    assert!(crate::read_outbox(&f.path).unwrap().is_empty());
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
}

#[test]
fn gone_endpoint_is_durably_retired() {
    let f = Fixture::new(1);
    let current = f.read();
    push_finish_batch(
        &f.state,
        vec![(
            f.sub.id.clone(),
            current,
            Some(f.sub.clone()),
            Err(PushFailure::Gone),
        )],
    )
    .unwrap();
    assert!(
        !f.path
            .join("state/push-subs")
            .join(format!("{}.json", f.sub.id))
            .exists()
    );
    assert!(f.state.data.lock().unwrap().push.subscriptions.is_empty());
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
}

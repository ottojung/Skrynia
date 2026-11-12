use super::fixture::Fixture;
use crate::{Notification, push_finish};
#[test]
fn newer_generation_survives_stale_acknowledgment() {
    let f = Fixture::new(2);
    let stale = Notification {
        subscription_id: f.sub.id.clone(),
        generation: 1,
        attempts: 0,
        next_at: 0,
    };
    push_finish(&f.state, &stale, Some(&f.sub), Ok(())).unwrap();
    assert_eq!(f.read().generation, 2);
    let latest = f.read();
    push_finish(&f.state, &latest, Some(&f.sub), Ok(())).unwrap();
    assert!(
        !f.path
            .join("state/push/outbox")
            .join(format!("{}.json", f.sub.id))
            .exists()
    );
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
}

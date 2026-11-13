use super::Fixture;
use crate::push_finish;
#[test]
fn successful_ack_removes_snapshot_only_notification_durably() {
    let f = Fixture::new(1);
    let pending = f.read();
    push_finish(&f.state, &pending, Some(&f.sub), Ok(())).unwrap();
    assert!(f.state.data.lock().unwrap().push.outbox.is_empty());
    assert!(crate::read_outbox(&f.path).unwrap().is_empty());
}

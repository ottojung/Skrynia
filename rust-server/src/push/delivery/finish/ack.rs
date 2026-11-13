use crate::Snapshot;
pub fn remove(data: &mut Snapshot, id: &str) {
    data.push.outbox.remove(id);
}

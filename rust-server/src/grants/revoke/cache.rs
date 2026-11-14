use crate::Snapshot;
pub fn rebuild(data: &mut Snapshot) {
    data.quota = data.quota.rebuild_live(&data.grants, &data.push);
}

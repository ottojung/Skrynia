use crate::{Grant, Snapshot, Usage};
pub fn attach(data: &mut Snapshot, grant: &Grant, totals: Option<Usage>) {
    if let Some(total) = totals {
        data.quota.scopes.insert(&grant.locations, total);
    }
    data.quota.attach_grant(&grant.id, &grant.locations);
}

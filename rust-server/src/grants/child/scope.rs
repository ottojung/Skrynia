use crate::{Grant, Result, Snapshot, State, Usage};
pub fn initial(state: &State, data: &Snapshot, scope: &str) -> Result<Option<Usage>> {
    if data.quota.scopes.has(scope) {
        return Ok(None);
    }
    let totals =
        crate::QuotaIndex::scope_usage(&state.dir, scope).map_err(|_| crate::internal())?;
    Ok(Some(totals))
}
pub fn attach(data: &mut Snapshot, grant: &Grant, totals: Option<Usage>) {
    if let Some(totals) = totals {
        data.quota.scopes.insert(&grant.locations, totals);
    }
    data.quota.attach_grant(&grant.id, &grant.locations);
}

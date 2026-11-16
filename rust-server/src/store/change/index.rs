use crate::{Result, Snapshot, State};
use std::sync::atomic::Ordering;
pub fn record(
    state: &State,
    data: &mut Snapshot,
    key: &str,
    old: Option<u64>,
    new: Option<u64>,
) -> Result<()> {
    data.bump_revision();
    data.quota.scopes.change(key, old, new);
    if let Some(index) = &data.quota.delta {
        let mut index = index.lock().map_err(|_| crate::internal())?;
        if let Err(error) = index.change(key, old, new) {
            state.failed.store(true, Ordering::Release);
            eprintln!("Skrynia derived quota index recovery required: {error}");
            return Err(crate::internal());
        }
    }
    Ok(())
}

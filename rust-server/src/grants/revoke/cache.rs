use crate::{Result, Snapshot, State};
use std::sync::atomic::Ordering;
pub fn reload(state: &State, data: &mut Snapshot) -> Result<()> {
    match crate::QuotaIndex::from_disk(&state.dir, &data.grants, &data.push) {
        Ok(index) => {
            data.quota = index;
            Ok(())
        }
        Err(error) => {
            state.failed.store(true, Ordering::Release);
            eprintln!("Skrynia revocation quota rebuild failed: {error}");
            Err(crate::internal())
        }
    }
}

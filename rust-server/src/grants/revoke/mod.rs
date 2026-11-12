mod disk;
mod record;
mod recover;
use crate::{Result, State, write_json};
use std::sync::atomic::Ordering;

pub use recover::recover_pending;
impl State {
    pub fn revoke_grant(&self, id: &str, token: &str) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        data.issuer(id, token)?;
        let ids = data
            .children_of(id)
            .iter()
            .map(|g| g.id.clone())
            .collect::<Vec<_>>();
        let record = record::Revocation { id: id.into(), ids };
        let marker = self
            .dir
            .join("state/grant-revocations")
            .join(format!("{id}.json"));
        write_json(&marker, &record).map_err(|_| crate::internal())?;
        if let Err(error) = disk::apply(&self.dir, &marker, &record) {
            self.failed.store(true, Ordering::Release);
            eprintln!("Skrynia revocation recovery required: {error}");
            return Err(crate::internal());
        }
        for id in record.ids {
            if let Some(grant) = data.grants.remove(&id) {
                for user in grant.users {
                    data.tokens.remove(&user.hash);
                }
            }
        }
        Ok(())
    }
}

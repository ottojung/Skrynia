mod disk;
mod record;
mod recover;
use crate::{Result, State};
use std::sync::atomic::Ordering;

pub use recover::recover_pending;
impl State {
    pub fn revoke_grant(&self, id: &str, token: &str) -> Result<()> {
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        self.require_healthy()?;
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
        self.persist_json(&marker, &record)?;
        if let Err(error) = disk::apply(&self.dir, &marker, &record) {
            self.failed.store(true, Ordering::Release);
            eprintln!("Skrynia revocation recovery required: {error}");
            return Err(crate::internal());
        }
        let obsolete = data
            .push
            .subscriptions
            .values()
            .filter(|sub| record.ids.contains(&sub.grant_id))
            .map(|sub| sub.id.clone())
            .collect::<Vec<_>>();
        for id in obsolete {
            data.push.subscriptions.remove(&id);
            data.push.outbox.remove(&id);
        }
        self.fail_on_io(crate::sync_outbox_index(&data, &self.dir))?;
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

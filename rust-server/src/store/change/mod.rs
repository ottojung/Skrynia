mod apply;
mod check;
mod failure;
mod lock;
use super::staging;
use super::{State, journal};
use crate::Result;

impl State {
    pub fn change(
        &self,
        key: &str,
        kind: &str,
        body: &[u8],
        expected: Option<&str>,
        token: &str,
    ) -> Result<bool> {
        // Preflight before I/O, then revalidate after parallel staging.
        drop(lock::checked(
            self, key, kind, body, expected, token, false,
        )?);
        let staged = if kind == "delete" {
            None
        } else {
            Some(
                staging::Staged::prepare(&self.dir, body)
                    .map_err(|e| crate::error::transient_io(&e))?,
            )
        };
        let mut data = lock::checked(self, key, kind, body, expected, token, true)?;
        let path = self.dir.join("store").join(key);
        let digest = (kind != "delete").then(|| crate::sha(body));
        let pending = journal::commit(
            &self.dir,
            key,
            kind,
            digest,
            || apply::commit(&self.dir, &path, kind, staged.as_ref()),
            || crate::push_enqueue(&mut data, &self.dir, key, kind, self.push_config.max_outbox),
        )
        .map_err(|failure| failure::map(self, failure))?;
        if kind == "delete" {
            data.objects.remove(key);
        } else {
            data.objects.insert(key.into(), body.len() as u64);
        }
        Ok(pending)
    }
}

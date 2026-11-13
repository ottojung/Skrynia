mod apply;
mod check;
mod lock;
use super::staging;
use super::{State, journal};
use crate::Result;
use std::sync::atomic::Ordering;

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
        drop(lock::checked(self, key, kind, body, expected, token)?);
        let staged = if kind == "delete" {
            None
        } else {
            Some(self.fail_on_io(staging::Staged::prepare(&self.dir, body))?)
        };
        let mut data = lock::checked(self, key, kind, body, expected, token)?;
        let path = self.dir.join("store").join(key);
        let digest = if kind == "delete" {
            None
        } else {
            Some(crate::sha(body))
        };
        let pending = journal::commit(
            &self.dir,
            key,
            kind,
            digest,
            || apply::commit(&self.dir, &path, kind, staged.as_ref()),
            || crate::push_enqueue(&mut data, &self.dir, key, kind, self.push_config.max_outbox),
        )
        .map_err(|error| {
            self.failed.store(true, Ordering::Release);
            eprintln!("Skrynia storage recovery required: {error}");
            crate::internal()
        })?;
        if kind == "delete" {
            data.objects.remove(key);
        } else {
            data.objects.insert(key.into(), body.len() as u64);
        }
        Ok(pending)
    }
}

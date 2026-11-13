mod apply;
mod check;
use super::{State, journal};
use super::{disk, staging};
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
        let mut data = self.data.lock().map_err(|_| crate::internal())?;
        self.require_healthy()?;
        let path = self.dir.join("store").join(key);
        check::preflight(&data, key, kind, body, expected, &path, token)?;
        crate::push_preflight(&data, key, kind, self.push_config.max_outbox)?;
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
            || apply::commit(&self.dir, &path, kind, body),
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

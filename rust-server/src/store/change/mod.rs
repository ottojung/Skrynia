mod apply;
mod check;
mod failure;
mod kind;
mod lock;
mod prepare;
use super::{State, journal, staging};
use crate::Result;
pub use kind::Kind;

impl State {
    pub fn change(
        &self,
        key: &str,
        kind: Kind,
        body: &[u8],
        expected: Option<&str>,
        token: &str,
    ) -> Result<bool> {
        // Preflight before I/O, then revalidate after parallel staging.
        let staged = prepare::staged(self, key, kind, body, token)?;
        let mut data = lock::checked(self, key, kind, body, token, check::Mode::Final(expected))?;
        let path = self.dir.join("store").join(key);
        let digest = (kind != Kind::Delete).then(|| crate::sha(body));
        let pending = journal::commit(
            &self.dir,
            key,
            kind.event(),
            digest,
            || apply::commit(&self.dir, &path, kind, staged.as_ref()),
            || {
                crate::push_enqueue(
                    &mut data,
                    &self.dir,
                    key,
                    kind.event(),
                    self.push_config.max_outbox,
                )
            },
        )
        .map_err(|failure| failure::map(self, failure))?;
        if kind == Kind::Delete {
            data.objects.remove(key);
        } else {
            data.objects.insert(key.into(), body.len() as u64);
        }
        Ok(pending)
    }
}

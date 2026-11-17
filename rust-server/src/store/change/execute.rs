use super::{Kind, apply, check, failure, index, lock};
use crate::{Result, State, Upload};
impl State {
    pub fn change_uploaded(
        &self,
        key: &str,
        kind: Kind,
        upload: &Upload,
        expected: Option<&str>,
        token: &str,
    ) -> Result<bool> {
        let (mut data, old) = lock::checked(
            self,
            key,
            kind,
            upload.size,
            token,
            check::Mode::Final(expected),
        )?;
        let path = self.dir.join("store").join(key);
        let pending = crate::commit_store(
            &self.dir,
            key,
            kind.event(),
            upload.digest.clone(),
            || apply::commit(&self.dir, &path, kind, upload.staged.as_ref()),
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
        index::record(
            self,
            &mut data,
            key,
            old,
            (kind != Kind::Delete).then_some(upload.size),
        )?;
        Ok(pending)
    }
}

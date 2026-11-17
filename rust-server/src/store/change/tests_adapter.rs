use super::Kind;
use crate::{Result, Staged, State, Upload};
use std::io::Write;
impl State {
    pub fn change(
        &self,
        key: &str,
        kind: Kind,
        body: &[u8],
        expected: Option<&str>,
        token: &str,
    ) -> Result<bool> {
        drop(super::lock::checked(
            self,
            key,
            kind,
            body.len() as u64,
            token,
            super::check::Mode::Initial,
        )?);
        let staged = if kind == Kind::Delete {
            None
        } else {
            let (stage, mut file) = Staged::start(&self.dir).map_err(|_| crate::internal())?;
            file.write_all(body).map_err(|_| crate::internal())?;
            file.sync_all().map_err(|_| crate::internal())?;
            Some(stage)
        };
        let upload = Upload {
            staged,
            size: body.len() as u64,
            digest: (kind != Kind::Delete).then(|| crate::sha(body)),
        };
        self.change_uploaded(key, kind, &upload, expected, token)
    }
}

use super::disk;
mod commit;
mod intent;
mod recover;

use super::State;
pub use commit::commit;
pub use recover::recover;
use std::io;
impl State {
    pub fn recover_journal(&self) -> io::Result<()> {
        let mut data = self.data.lock().map_err(|_| io::Error::other("poisoned"))?;
        recover(&self.dir, |intent| {
            crate::push_enqueue(
                &mut data,
                &self.dir,
                &intent.key,
                &intent.kind,
                self.push_config.max_outbox,
            )
        })
    }
}

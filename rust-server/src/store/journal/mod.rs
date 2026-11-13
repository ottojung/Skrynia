use super::disk;
mod commit;
mod intent;
mod recover;

use super::State;
pub use commit::commit;
pub use recover::recover;
use std::io;
use std::sync::atomic::Ordering;
impl State {
    pub fn recover_journal(&self) -> io::Result<()> {
        let mut data = self.data.lock().map_err(|_| io::Error::other("poisoned"))?;
        if self.failed.load(Ordering::Acquire) {
            return Err(io::Error::other("recovery required"));
        }
        let result = recover(&self.dir, |intent| {
            crate::push_enqueue(
                &mut data,
                &self.dir,
                &intent.key,
                &intent.kind,
                self.push_config.max_outbox,
            )
        });
        if result.is_err() {
            self.failed.store(true, Ordering::Release);
        }
        result
    }
}

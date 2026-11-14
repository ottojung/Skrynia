use crate::{Pending, PushState};
impl PushState {
    pub fn retry_attempts(&self, id: &str, pending: &Pending) -> u32 {
        self.cooldowns
            .get(id)
            .filter(|c| c.generation == pending.generation)
            .map_or(pending.attempts, |c| {
                pending.attempts.max(c.delivery_errors)
            })
    }
    pub fn effective_retry(&self, id: &str, pending: &Pending) -> (u32, u64) {
        match self
            .cooldowns
            .get(id)
            .filter(|c| c.generation == pending.generation)
        {
            Some(c) if c.delivery_errors > pending.attempts => {
                (c.delivery_errors, pending.next_at.max(c.until))
            }
            _ => (pending.attempts, pending.next_at),
        }
    }
    pub fn ready(&self, id: &str, pending: &Pending, now: u64) -> bool {
        pending.next_at <= now
            && self
                .cooldowns
                .get(id)
                .filter(|c| c.generation == pending.generation)
                .is_none_or(|c| c.until <= now)
    }
    pub fn prune_cooldowns(&mut self) {
        self.cooldowns.retain(|id, c| {
            self.outbox
                .get(id)
                .is_some_and(|p| p.generation == c.generation)
        });
    }
}

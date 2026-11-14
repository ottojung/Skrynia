use super::Cooldown;
use crate::PushState;
impl PushState {
    pub fn delay_failed_ack(&mut self, id: &str, delivery_failed: bool) {
        let Some(pending) = self.outbox.get(id) else {
            return;
        };
        let cooldown = self
            .cooldowns
            .entry(id.into())
            .or_insert_with(|| Cooldown::new(pending));
        if cooldown.generation != pending.generation {
            *cooldown = Cooldown::new(pending);
        }
        cooldown.ack_errors = cooldown.ack_errors.saturating_add(1);
        if delivery_failed {
            cooldown.delivery_errors = cooldown
                .delivery_errors
                .max(pending.attempts)
                .saturating_add(1);
        }
        let attempts = cooldown.ack_errors.max(cooldown.delivery_errors);
        cooldown.until = crate::push_now().saturating_add(crate::push_delay(attempts));
    }
    pub fn clear_cooldown(&mut self, id: &str) {
        self.cooldowns.remove(id);
    }
}

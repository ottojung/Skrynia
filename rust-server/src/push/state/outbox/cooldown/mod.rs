mod lookup;
mod schedule;
use crate::Pending;
#[derive(Clone, Copy)]
pub struct Cooldown {
    generation: u64,
    ack_errors: u32,
    delivery_errors: u32,
    until: u64,
}
impl Cooldown {
    fn new(pending: &Pending) -> Self {
        Self {
            generation: pending.generation,
            ack_errors: 0,
            delivery_errors: pending.attempts,
            until: 0,
        }
    }
}

#[cfg(test)]
mod tests;

use super::{Cache, flight::Flight};
use std::sync::Arc;

pub(super) enum Claim {
    Ready(wasmtime::Module),
    Wait(Arc<Flight>),
    Compile(Arc<Flight>),
}

impl Cache {
    pub(super) fn claim(&self, key: [u8; 32]) -> Claim {
        let mut active = self.inflight.lock().unwrap();
        // Recheck after the optimistic lookup. A previous leader may have
        // published and removed its flight while this caller was suspended.
        if let Some(m) = self.ready.lock().unwrap().get(&key).cloned() {
            return Claim::Ready(m);
        }
        if let Some(f) = active.get(&key) {
            return Claim::Wait(f.clone());
        }
        let f = Arc::new(Flight::new());
        active.insert(key, f.clone());
        Claim::Compile(f)
    }
}

#[cfg(test)]
mod tests;

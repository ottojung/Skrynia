use super::{Cache, MAX_ENTRIES, claim::Claim, guard::Guard};
use std::sync::Arc;

impl Cache {
    pub fn compile<F>(&self, code: &[u8], f: F) -> Result<wasmtime::Module, &'static str>
    where
        F: FnOnce() -> Result<wasmtime::Module, &'static str>,
    {
        let key = Self::key(code);
        if let Some(m) = self.ready.lock().unwrap().get(&key).cloned() {
            return Ok(m);
        }
        let flight = match self.claim(key) {
            Claim::Ready(m) => return Ok(m),
            Claim::Wait(f) => return f.wait(),
            Claim::Compile(f) => f,
        };
        let mut g = Guard {
            map: &self.inflight,
            key,
            flight: flight.clone(),
            done: false,
        };
        let r = f();
        if let Ok(m) = &r {
            let mut rm = self.ready.lock().unwrap();
            if rm.len() >= MAX_ENTRIES {
                rm.clear();
            }
            rm.insert(key, m.clone());
        }
        flight.finish(r.clone());
        let mut im = self.inflight.lock().unwrap();
        if matches!(im.get(&key), Some(f) if Arc::ptr_eq(f, &flight)) {
            im.remove(&key);
        }
        drop(im);
        g.done = true;
        r
    }
}

use super::{Cache, MAX_ENTRIES, flight::Flight, guard::Guard};
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
        let flight = {
            let mut m = self.inflight.lock().unwrap();
            if let Some(f) = m.get(&key) {
                let f = f.clone();
                drop(m);
                return f.wait();
            }
            let f = Arc::new(Flight::new());
            m.insert(key, f.clone());
            f
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
        let mut im = self.inflight.lock().unwrap();
        if matches!(im.get(&key), Some(f) if Arc::ptr_eq(f, &flight)) {
            im.remove(&key);
        }
        drop(im);
        flight.finish(r.clone());
        g.done = true;
        r
    }
}

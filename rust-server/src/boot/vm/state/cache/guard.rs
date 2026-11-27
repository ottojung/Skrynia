use super::flight::Flight;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
pub struct Guard<'a> {
    pub map: &'a Mutex<HashMap<[u8; 32], Arc<Flight>>>,
    pub key: [u8; 32],
    pub flight: Arc<Flight>,
    pub done: bool,
}
impl Drop for Guard<'_> {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        self.flight.finish(Err("limit"));
        let mut m = self.map.lock().unwrap_or_else(|e| e.into_inner());
        if matches!(m.get(&self.key), Some(f) if Arc::ptr_eq(f, &self.flight)) {
            m.remove(&self.key);
        }
    }
}

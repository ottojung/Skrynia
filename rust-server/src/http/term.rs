use std::io;
use std::sync::{Arc, Mutex};

pub type Term = Arc<Mutex<Option<(io::ErrorKind, String)>>>;

pub fn new() -> Term {
    Arc::new(Mutex::new(None))
}
pub fn set(t: &Term, e: &io::Error) {
    *t.lock().unwrap_or_else(|p| p.into_inner()) = Some((e.kind(), e.to_string()));
}
pub fn get(t: &Term) -> Option<io::Error> {
    t.lock()
        .unwrap_or_else(|p| p.into_inner())
        .as_ref()
        .map(|(k, m)| io::Error::new(*k, m.as_str()))
}

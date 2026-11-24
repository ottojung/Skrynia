mod config;
mod http;
mod link;
mod names;
mod optimization;

pub use optimization::EquivalentCall;
use std::collections::BTreeMap;

/// One complete execution of the syscall HTTP prescription.
#[derive(Default)]
pub struct Syscalls {
    calls: BTreeMap<String, EquivalentCall>,
}

impl Syscalls {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn names(&self) -> Vec<String> {
        self.calls.keys().cloned().collect()
    }
}

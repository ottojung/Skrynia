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

    pub(crate) fn signatures(&self) -> std::collections::BTreeMap<String, crate::Signature> {
        self.calls
            .iter()
            .map(|(name, call)| (name.clone(), call.signature.clone()))
            .collect()
    }
}

impl Syscalls {
    /// Replace the HTTP transport with equivalent native implementations.
    pub fn optimized(calls: impl IntoIterator<Item = EquivalentCall>) -> Result<Self, String> {
        let mut result = Self::empty();
        for call in calls {
            let name = call.name.clone();
            if result.calls.insert(name.clone(), call).is_some() {
                return Err(format!("duplicate syscall: {name}"));
            }
        }
        Ok(result)
    }
}

pub(crate) fn check_name(name: &str) -> Result<(), String> {
    names::check(name)
}

use super::{EquivalentCall, Signature};
use std::sync::Arc;
use wasmtime::Val;

pub(super) type Context = Arc<
    dyn for<'a> Fn(
            &mut wasmtime::Caller<'a, crate::boot::vm::state::Vm>,
            &[Val],
        ) -> Result<Vec<Val>, String>
        + Send
        + Sync,
>;

impl EquivalentCall {
    /// Native implementation with access to the calling machine context.
    pub fn with_context(
        name: &str,
        params: &str,
        results: &str,
        call: impl for<'a> Fn(
            &mut wasmtime::Caller<'a, crate::boot::vm::state::Vm>,
            &[Val],
        ) -> Result<Vec<Val>, String>
        + Send
        + Sync
        + 'static,
    ) -> Result<Self, String> {
        let signature = Signature::new(params, results)?;
        let link = super::link::context(signature.clone(), Arc::new(call))?;
        Self::build(name, signature, link)
    }
}

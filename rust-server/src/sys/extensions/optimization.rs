mod context;
mod link;

use super::check_name;
use crate::Signature;
use std::sync::Arc;
use wasmtime::Val;

pub(super) type Native = Arc<dyn Fn(&[Val]) -> Result<Vec<Val>, String> + Send + Sync>;

/// A transport-free native equivalent of one typed POST endpoint.
pub struct EquivalentCall {
    pub(super) name: String,
    pub(super) signature: Signature,
    pub(super) link: link::Link,
}

impl EquivalentCall {
    pub fn new(
        name: &str,
        params: &str,
        results: &str,
        call: impl Fn(&[Val]) -> Result<Vec<Val>, String> + Send + Sync + 'static,
    ) -> Result<Self, String> {
        let signature = Signature::new(params, results)?;
        let native: Native = Arc::new(call);
        let link = link::native(signature.clone(), native)?;
        Self::build(name, signature, link)
    }

    fn build(name: &str, signature: Signature, link: link::Link) -> Result<Self, String> {
        check_name(name)?;
        Ok(Self {
            name: name.into(),
            signature,
            link,
        })
    }
}

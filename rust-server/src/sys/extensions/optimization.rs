type Link = Box<
    dyn FnOnce(&mut wasmtime::Linker<crate::boot::vm::state::Vm>, &str) -> Result<(), String>
        + Send,
>;

/// A transport-free equivalent of one POST endpoint.
/// The caller is responsible for preserving its full observable semantics.
pub struct EquivalentCall {
    pub(super) name: String,
    pub(super) link: Link,
}

impl EquivalentCall {
    pub fn new(
        name: &str,
        call: impl Fn(u32) -> Result<u32, String> + Send + Sync + 'static,
    ) -> Result<Self, String> {
        super::names::check(name)?;
        Ok(Self {
            name: name.into(),
            link: Box::new(move |linker, name| {
                linker
                    .func_wrap("syscall", name, move |arg: u32| {
                        call(arg).map_err(wasmtime::Error::msg)
                    })
                    .map_err(|e| e.to_string())?;
                Ok(())
            }),
        })
    }
}

impl super::Syscalls {
    /// Replace the entire HTTP prescription with its equivalent operations.
    /// The supplied names must equal the reference server's discovery list.
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

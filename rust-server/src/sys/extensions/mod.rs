mod config;
mod http;
mod link;
mod names;

type Registration = Box<
    dyn FnOnce(&mut wasmtime::Linker<crate::boot::vm::state::Vm>, &str) -> Result<(), String>
        + Send,
>;

/// Operator-selected host functions. Only Program 0 can import them.
#[derive(Default)]
pub struct Syscalls {
    native: std::collections::BTreeMap<String, Registration>,
    fallback: Option<http::Fallback>,
}

impl Syscalls {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a trusted native implementation with explicit global RAM access.
    pub fn register<F>(&mut self, name: &str, function: F) -> Result<(), String>
    where
        F: Fn(&crate::ram::Ram, u32) -> Result<u32, String> + Send + Sync + 'static,
    {
        names::check(name)?;
        if self.native.contains_key(name) {
            return Err(format!("duplicate syscall: {name}"));
        }
        self.native.insert(
            name.to_owned(),
            Box::new(move |linker, name| {
                linker
                    .func_wrap(
                        "syscall",
                        name,
                        move |c: wasmtime::Caller<'_, crate::boot::vm::state::Vm>, arg: u32| {
                            function(&c.data().ram, arg).map_err(wasmtime::Error::msg)
                        },
                    )
                    .map_err(|e| e.to_string())?;
                Ok(())
            }),
        );
        Ok(())
    }

    pub fn names(&self) -> Vec<String> {
        let mut names: std::collections::BTreeSet<_> = self.native.keys().cloned().collect();
        if let Some(fallback) = &self.fallback {
            names.extend(fallback.names.iter().cloned());
        }
        names.into_iter().collect()
    }
}

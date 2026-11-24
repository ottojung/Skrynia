impl super::Syscalls {
    pub(crate) fn link(
        mut self,
        linker: &mut wasmtime::Linker<crate::boot::vm::state::Vm>,
        module: &wasmtime::Module,
    ) -> Result<(), String> {
        let imports: std::collections::BTreeSet<_> = module
            .imports()
            .filter(|i| i.module() == "syscall")
            .map(|i| i.name().to_owned())
            .collect();
        for name in imports {
            if let Some(register) = self.native.remove(&name) {
                register(linker, &name)?;
            } else if let Some(fallback) = &self.fallback
                && fallback.names.contains(&name)
            {
                fallback.link(linker, &name)?;
            } else {
                return Err(format!("unsupported syscall: {name}"));
            }
        }
        Ok(())
    }
}

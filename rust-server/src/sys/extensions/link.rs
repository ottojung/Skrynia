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
            let equivalent = self
                .calls
                .remove(&name)
                .ok_or_else(|| format!("unsupported syscall: {name}"))?;
            (equivalent.link)(linker, &name)?;
        }
        Ok(())
    }
}

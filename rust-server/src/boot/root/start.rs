impl crate::boot::Kernel {
    pub fn start_with_syscalls(
        root_wasm: &std::path::Path,
        operator_dir: &std::path::Path,
        user_dir: &std::path::Path,
        syscalls: crate::Syscalls,
    ) -> Result<Self, String> {
        let cfg = crate::boot::limits::engine_config()?;
        let engine = wasmtime::Engine::new(&cfg).map_err(|e| format!("engine: {e}"))?;
        let bytes = super::bytes(root_wasm)?;
        let root = crate::boot::vm::compile::trusted(&engine, &bytes)
            .map_err(|e| format!("compile root: {e}"))?;
        crate::boot::validate::check(&root)?;
        let signatures = syscalls.signatures();
        crate::boot::validate::check_syscalls(&root, &signatures)?;
        let version = crate::sys::version::info(&signatures)?;
        let mut linker = wasmtime::Linker::new(&engine);
        crate::boot::vm::link::root::add(&mut linker)?;
        syscalls.link(&mut linker, &root)?;
        let pre = linker.instantiate_pre(&root).map_err(|e| e.to_string())?;
        let operator = std::sync::Arc::new(crate::fs::root::Root::open_unhardened(operator_dir)?);
        let user = std::sync::Arc::new(crate::fs::root::Root::open_unhardened(user_dir)?);
        crate::boot::distinct::ensure_distinct(&operator, &user, operator_dir, user_dir)?;
        operator.restrict_permissions()?;
        user.restrict_permissions()?;
        let ram = std::sync::Arc::new(crate::ram::Ram::new());
        let fs = std::sync::Arc::new(crate::fs::Fs::new(operator, user, ram.clone()));
        let cache = std::sync::Arc::new(crate::boot::vm::state::Cache::new());
        Ok(Self {
            engine,
            root,
            ram,
            fs,
            cache,
            pre,
            version,
        })
    }
}

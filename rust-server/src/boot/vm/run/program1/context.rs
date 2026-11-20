pub struct Context<'a> {
    pub engine: &'a wasmtime::Engine,
    pub module: &'a wasmtime::Module,
    pub fs: std::sync::Arc<crate::fs::Fs>,
    pub ram: std::sync::Arc<crate::ram::Ram>,
    pub cache: std::sync::Arc<crate::boot::vm::state::Cache>,
    pub scoped: bool,
}

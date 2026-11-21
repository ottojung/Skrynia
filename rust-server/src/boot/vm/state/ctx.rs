#[derive(Clone)]
pub struct ProviderCtx {
    pub inst: wasmtime::Instance,
    pub guard: Option<super::GuestCap>,
    pub eval: bool,
}

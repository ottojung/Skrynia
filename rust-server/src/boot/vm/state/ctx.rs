#[derive(Clone)]
pub struct ProviderCtx {
    pub inst: wasmtime::Instance,
    pub guard: Option<super::GuestCap>,
    pub eval: bool,
    pub fs_auth: crate::boot::vm::cap::FsAuth,
    pub fs_delegate: crate::boot::vm::cap::FsAuth,
}

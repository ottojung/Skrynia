impl super::Syscalls {
    /// Reference execution: discover and execute through the HTTP server.
    pub fn http(base: &str) -> Result<Self, String> {
        super::http::Provider::discover(base)?.into_calls()
    }

    /// Reference execution from the operator's environment.
    pub fn from_env() -> Result<Self, String> {
        match std::env::var("SKRYNIA_SYSCALL_URL") {
            Ok(base) => Self::http(&base),
            Err(std::env::VarError::NotPresent) => Ok(Self::empty()),
            Err(_) => Err("invalid SKRYNIA_SYSCALL_URL".into()),
        }
    }
}

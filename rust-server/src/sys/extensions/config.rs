impl super::Syscalls {
    /// Configure the HTTP provider by discovering names from <base>/version.
    pub fn http_provider(&mut self, base: &str) -> Result<(), String> {
        if self.fallback.is_some() {
            return Err("HTTP syscall provider already configured".into());
        }
        self.fallback = Some(super::http::Fallback::discover(base)?);
        Ok(())
    }

    /// Configure the HTTP provider from SKRYNIA_SYSCALL_URL when present.
    pub fn configure_env(&mut self) -> Result<(), String> {
        if let Some(base) = optional("SKRYNIA_SYSCALL_URL")? {
            self.http_provider(&base)?;
        }
        Ok(())
    }
}

fn optional(key: &str) -> Result<Option<String>, String> {
    match std::env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(format!("invalid {key}")),
    }
}

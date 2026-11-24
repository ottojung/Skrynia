impl super::Syscalls {
    /// Configure the HTTP implementation for an explicit set of names.
    pub fn http_fallback(&mut self, base: &str, names: &[&str]) -> Result<(), String> {
        if self.fallback.is_some() {
            return Err("HTTP syscall fallback already configured".into());
        }
        self.fallback = Some(super::http::Fallback::new(base, names)?);
        Ok(())
    }

    pub fn configure_env(&mut self) -> Result<(), String> {
        let base = optional("SKRYNIA_SYSCALL_URL")?;
        let names = optional("SKRYNIA_SYSCALLS")?;
        match (base, names) {
            (None, None) => Ok(()),
            (Some(base), Some(names)) => {
                let names: Vec<_> = names.split(',').map(str::trim).collect();
                self.http_fallback(&base, &names)
            }
            _ => Err("set both SKRYNIA_SYSCALL_URL and SKRYNIA_SYSCALLS".into()),
        }
    }
}

fn optional(key: &str) -> Result<Option<String>, String> {
    match std::env::var(key) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(_) => Err(format!("invalid {key}")),
    }
}
